//! The gateway (PROTOCOL §8): WS at `/api/v1/gateway`.
//!
//! Shape: one broadcast bus of [`ServerEvent`]s; a *session task* per session
//! that assigns per-session sequence numbers, keeps the 500-frame resume ring,
//! and forwards to whichever socket is currently attached. The session task
//! outlives its socket by up to 120s (`RESUME_WINDOW_MS`) so a reconnecting
//! client can replay what it missed — no gaps, no duplicates.
//!
//! Presence lives in memory only, on purpose (ARCHITECTURE §5): restart ⇒
//! everyone offline until they reconnect.

mod socket;

use std::collections::VecDeque;
use std::sync::Arc;
use std::time::{Duration, Instant};

use dashmap::DashMap;
use linger_core::gateway::{ServerEvent, ServerFrame};
use linger_core::limits::{RESUME_BUFFER_FRAMES, RESUME_WINDOW_MS};
use linger_core::wire::{ActivityInfo, PresenceEntry, PresenceState};
use linger_core::{RoomId, UserId};
use tokio::sync::{broadcast, mpsc};

pub use socket::ws_route;

use crate::db::now_ms;

/// Everything the fan-out layer holds. Lives in `AppState` as `Arc<Gateway>`;
/// REST handlers publish mutation events through [`Gateway::publish`].
pub struct Gateway {
    bus: broadcast::Sender<Arc<ServerEvent>>,
    sessions: DashMap<String, SessionHandle>,
    presence: DashMap<UserId, PresenceEntry>,
    conn_count: DashMap<UserId, u32>,
    registry: linger_activity::registry::Registry,
}

struct SessionHandle {
    user_id: UserId,
    ctl: mpsc::Sender<Ctl>,
}

enum Ctl {
    Attach {
        sink: mpsc::Sender<String>,
        /// Last sequence number the client saw; replay starts after it.
        resume_from: u64,
        /// Resume attaches get a `resumed` frame before the replay.
        is_resume: bool,
    },
    Detach,
}

impl Default for Gateway {
    fn default() -> Self {
        Self::new()
    }
}

impl Gateway {
    #[must_use]
    pub fn new() -> Self {
        let (bus, _) = broadcast::channel(1024);
        let registry = linger_activity::registry::Registry::from_json(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../registry/apps.json"
        )))
        .expect("bundled registry parses (guarded by linger-activity tests)");
        Self {
            bus,
            sessions: DashMap::new(),
            presence: DashMap::new(),
            conn_count: DashMap::new(),
            registry,
        }
    }

    /// Fan an event out to every session. REST mutation handlers call this.
    pub fn publish(&self, event: ServerEvent) {
        // No receivers just means nobody is connected; that is fine.
        let _ = self.bus.send(Arc::new(event));
    }

    /// Presence snapshot for `ready`.
    #[must_use]
    pub fn presence_snapshot(&self) -> Vec<PresenceEntry> {
        self.presence.iter().map(|e| e.value().clone()).collect()
    }

    /// Who is in a room right now.
    #[must_use]
    pub fn occupancy(&self, room_id: RoomId) -> Vec<UserId> {
        self.presence
            .iter()
            .filter(|e| e.value().room_id == Some(room_id))
            .map(|e| *e.key())
            .collect()
    }

    /// Resolve a client-reported registry id. Unknown ids resolve to `None`:
    /// default deny holds server-side too, and the label/kind shown to friends
    /// always come from the bundled registry, never from client-supplied text.
    fn resolve_activity(&self, registry_id: Option<&str>, since: i64) -> Option<ActivityInfo> {
        let app = self.registry.get(registry_id?)?;
        Some(ActivityInfo {
            registry_id: app.id.clone(),
            label: app.label.clone(),
            kind: app.kind.clone(),
            since,
        })
    }

    /// Apply a client `presence.update`. Room membership only changes via
    /// `room.sit`, so it is carried over from the existing entry.
    fn apply_presence(
        &self,
        user_id: UserId,
        state: PresenceState,
        activity_id: Option<&str>,
        away_message: Option<String>,
    ) -> PresenceEntry {
        let prev = self.presence.get(&user_id).map(|e| e.value().clone());
        let room_id = prev.as_ref().and_then(|p| p.room_id);
        // Keep `since` stable while the same app stays foreground.
        let since = match (&prev.and_then(|p| p.activity), activity_id) {
            (Some(old), Some(new_id)) if old.registry_id == new_id => old.since,
            _ => now_ms(),
        };
        let entry = PresenceEntry {
            user_id,
            state,
            room_id,
            activity: self.resolve_activity(activity_id, since),
            away_message,
        };
        self.presence.insert(user_id, entry.clone());
        entry
    }

    /// Apply a `room.focus` (or a `None` = left the room) and emit the resulting
    /// events, in protocol order: leave → enter → occupancy → presence.
    fn apply_room_focus(
        &self,
        user_id: UserId,
        room_id: Option<RoomId>,
        entrance_sound: Option<String>,
    ) {
        let prev_room = self.presence.get(&user_id).and_then(|e| e.value().room_id);
        if prev_room == room_id {
            return;
        }

        let entry = {
            let mut entry = self
                .presence
                .get(&user_id)
                .map(|e| e.value().clone())
                .unwrap_or_else(|| offline_entry(user_id));
            entry.room_id = room_id;
            entry.state = if room_id.is_some() {
                PresenceState::InRoom
            } else {
                PresenceState::Around
            };
            self.presence.insert(user_id, entry.clone());
            entry
        };

        if let Some(prev) = prev_room {
            self.publish(ServerEvent::RoomLeave {
                room_id: prev,
                user_id,
            });
            self.publish(ServerEvent::RoomOccupancy {
                room_id: prev,
                user_ids: self.occupancy(prev),
            });
        }
        if let Some(new) = room_id {
            self.publish(ServerEvent::RoomEnter {
                room_id: new,
                user_id,
                entrance_sound,
            });
            self.publish(ServerEvent::RoomOccupancy {
                room_id: new,
                user_ids: self.occupancy(new),
            });
        }
        self.publish(ServerEvent::PresenceUpdate(entry));
    }

    /// A connection came up for this user. First connection ⇒ visible presence.
    fn connection_opened(&self, user_id: UserId) {
        let mut count = self.conn_count.entry(user_id).or_insert(0);
        *count += 1;
        if *count == 1 {
            let entry = self.apply_presence(user_id, PresenceState::Around, None, None);
            self.publish(ServerEvent::PresenceUpdate(entry));
        }
    }

    /// A connection went away. Last connection ⇒ offline (leaving any room).
    fn connection_closed(&self, user_id: UserId) {
        let remaining = {
            let mut count = self.conn_count.entry(user_id).or_insert(1);
            *count = count.saturating_sub(1);
            *count
        };
        if remaining == 0 {
            self.conn_count.remove(&user_id);
            self.apply_room_focus(user_id, None, None);
            self.presence.remove(&user_id);
            self.publish(ServerEvent::PresenceUpdate(offline_entry(user_id)));
        }
    }

    /// PROTOCOL §8 fan-out rules: everything goes everywhere except
    /// `room.enter`, which only reaches clients currently in that room.
    fn visible_to(&self, receiver: UserId, event: &ServerEvent) -> bool {
        match event {
            ServerEvent::RoomEnter { room_id, .. } => self
                .presence
                .get(&receiver)
                .is_some_and(|e| e.value().room_id == Some(*room_id)),
            _ => true,
        }
    }
}

fn offline_entry(user_id: UserId) -> PresenceEntry {
    PresenceEntry {
        user_id,
        state: PresenceState::Offline,
        room_id: None,
        activity: None,
        away_message: None,
    }
}

/// Spawn a session task; returns its control handle. See module docs for shape.
fn spawn_session(gateway: Arc<Gateway>, session_id: String, user_id: UserId) -> mpsc::Sender<Ctl> {
    let (ctl_tx, mut ctl_rx) = mpsc::channel::<Ctl>(16);
    let mut bus_rx = gateway.bus.subscribe();

    tokio::spawn(async move {
        let mut seq: u64 = 0;
        let mut ring: VecDeque<(u64, String)> = VecDeque::with_capacity(RESUME_BUFFER_FRAMES);
        let mut sink: Option<mpsc::Sender<String>> = None;
        // Starts "detached": if the handshake never attaches, the sweep reaps us.
        let mut detached_at: Option<Instant> = Some(Instant::now());
        let mut sweep = tokio::time::interval(Duration::from_secs(5));

        loop {
            tokio::select! {
                event = bus_rx.recv() => match event {
                    Ok(event) => {
                        if !gateway.visible_to(user_id, &event) {
                            continue;
                        }
                        seq += 1;
                        let frame = ServerFrame::sequenced((*event).clone(), seq);
                        let Ok(json) = serde_json::to_string(&frame) else { continue };
                        if ring.len() == RESUME_BUFFER_FRAMES {
                            ring.pop_front();
                        }
                        ring.push_back((seq, json.clone()));
                        if let Some(tx) = &sink {
                            // A full queue means a dead or pathologically slow
                            // socket; detach and let resume recover it.
                            if tx.try_send(json).is_err() {
                                sink = None;
                                detached_at = Some(Instant::now());
                            }
                        }
                    }
                    // Lagged: we dropped bus frames, so the ring has a hole and
                    // resume would silently skip events. Integrity over uptime:
                    // kill the session; the client re-identifies and refetches.
                    Err(broadcast::error::RecvError::Lagged(_)) => break,
                    Err(broadcast::error::RecvError::Closed) => break,
                },
                ctl = ctl_rx.recv() => match ctl {
                    Some(Ctl::Attach { sink: new_sink, resume_from, is_resume }) => {
                        let replay: Vec<String> = ring
                            .iter()
                            .filter(|(s, _)| *s > resume_from)
                            .map(|(_, j)| j.clone())
                            .collect();
                        if is_resume {
                            let resumed = ServerFrame::control(ServerEvent::Resumed {
                                replayed: replay.len() as u64,
                            });
                            if let Ok(json) = serde_json::to_string(&resumed) {
                                let _ = new_sink.send(json).await;
                            }
                        }
                        let mut ok = true;
                        for json in replay {
                            if new_sink.send(json).await.is_err() {
                                ok = false;
                                break;
                            }
                        }
                        if ok {
                            sink = Some(new_sink);
                            detached_at = None;
                        }
                    }
                    Some(Ctl::Detach) => {
                        sink = None;
                        detached_at = Some(Instant::now());
                    }
                    None => break,
                },
                _ = sweep.tick() => {
                    let expired = detached_at
                        .is_some_and(|t| t.elapsed() >= Duration::from_millis(RESUME_WINDOW_MS));
                    if expired {
                        break;
                    }
                }
            }
        }
        gateway.sessions.remove(&session_id);
    });

    ctl_tx
}
