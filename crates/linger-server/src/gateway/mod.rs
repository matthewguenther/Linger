//! The gateway (PROTOCOL §8): WS at `/api/v1/gateway`.
//!
//! Shape: one broadcast bus of [`Fanout`]s — an event plus who it is for; a
//! *session task* per session that assigns per-session sequence numbers, keeps
//! the 500-frame resume ring, and forwards to whichever socket is currently
//! attached. The session task outlives its socket by up to 120s
//! (`RESUME_WINDOW_MS`) so a reconnecting client can replay what it missed —
//! no gaps, no duplicates.
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
use linger_core::wire::{PresenceEntry, PresenceState};
use linger_core::{RoomId, UserId};
use tokio::sync::{broadcast, mpsc, oneshot};

pub use socket::ws_route;

/// Everything the fan-out layer holds. Lives in `AppState` as `Arc<Gateway>`;
/// REST handlers publish mutation events through [`Gateway::publish`].
pub struct Gateway {
    bus: broadcast::Sender<Arc<Fanout>>,
    sessions: DashMap<String, SessionHandle>,
    presence: DashMap<UserId, PresenceEntry>,
    conn_count: DashMap<UserId, u32>,
}

/// One event on the bus, plus who it is for.
///
/// Every session subscribes to the same broadcast channel, so an event meant
/// for one person still passes every session task on its way past. `to` is what
/// those tasks check: `None` is the ordinary case and reaches everybody, and
/// `Some(id)` reaches that person's sessions and nobody else's.
///
/// Addressing lives here rather than on the event because the event is the wire
/// type (PROTOCOL §8) and a `knock` frame must not tell its receiver who it was
/// addressed to — they already know, and the sender's copy would be a field
/// carrying nothing but a way to get the fan-out wrong.
struct Fanout {
    event: ServerEvent,
    to: Option<UserId>,
}

struct SessionHandle {
    user_id: UserId,
    ctl: mpsc::Sender<Ctl>,
}

enum Ctl {
    Attach {
        sink: mpsc::Sender<String>,
        /// Fired to make the attached socket's reader loop give up. The socket
        /// is the only thing that can end itself otherwise, and it spends its
        /// life parked on the next client frame, which a removed member is
        /// never going to send.
        closer: oneshot::Sender<()>,
        /// Last sequence number the client saw; replay starts after it.
        resume_from: u64,
        /// Resume attaches get a `resumed` frame before the replay.
        is_resume: bool,
    },
    Detach,
    /// This session's user is off the server (T-413). Say so, hang up, stop.
    Close,
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
        Self {
            bus,
            sessions: DashMap::new(),
            presence: DashMap::new(),
            conn_count: DashMap::new(),
        }
    }

    /// Fan an event out to every session. REST mutation handlers call this.
    pub fn publish(&self, event: ServerEvent) {
        // No receivers just means nobody is connected; that is fine.
        let _ = self.bus.send(Arc::new(Fanout { event, to: None }));
    }

    /// Fan an event out to one person's sessions and nobody else's (T-1101).
    ///
    /// All of them, not one: somebody signed in on a laptop and a desktop is
    /// one person, and a knock that landed on whichever machine happened to
    /// connect first would be a knock they never saw.
    pub fn publish_to(&self, user_id: UserId, event: ServerEvent) {
        let _ = self.bus.send(Arc::new(Fanout {
            event,
            to: Some(user_id),
        }));
    }

    /// Hang up on every session this person has open (T-413).
    ///
    /// Removal is only real once the socket is gone. The token check at
    /// identify happens once, at the start, so an already-open socket keeps
    /// receiving every message on the server until somebody closes it — and
    /// waiting for the 15-minute access token to lapse is not closing it.
    pub async fn close_sessions_for(&self, user_id: UserId) {
        // Collected first: holding a `DashMap` iterator across an await is how
        // a map like this deadlocks against the session task removing itself.
        let open: Vec<mpsc::Sender<Ctl>> = self
            .sessions
            .iter()
            .filter(|e| e.value().user_id == user_id)
            .map(|e| e.value().ctl.clone())
            .collect();
        for ctl in open {
            let _ = ctl.send(Ctl::Close).await;
        }
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

    /// Apply a client `presence.update`. Room membership only changes via
    /// `room.focus`, so it is carried over from the existing entry.
    fn apply_presence(
        &self,
        user_id: UserId,
        state: PresenceState,
        away_message: Option<String>,
    ) -> PresenceEntry {
        let prev = self.presence.get(&user_id).map(|e| e.value().clone());
        let room_id = prev.as_ref().and_then(|p| p.room_id);
        let entry = PresenceEntry {
            user_id,
            state,
            room_id,
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
            let entry = self.apply_presence(user_id, PresenceState::Around, None);
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

    /// PROTOCOL §8 fan-out rules. Two things narrow an event: an addressed
    /// event reaches only the person it names, and `room.enter` reaches only
    /// clients currently in that room.
    fn visible_to(&self, receiver: UserId, fanout: &Fanout) -> bool {
        if let Some(target) = fanout.to {
            return target == receiver;
        }
        match &fanout.event {
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
        // Held alongside the sink and replaced with it: it belongs to whichever
        // socket is attached right now, not to the session.
        let mut closer: Option<oneshot::Sender<()>> = None;
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
                        let frame = ServerFrame::sequenced(event.event.clone(), seq);
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
                                closer = None;
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
                    Some(Ctl::Attach { sink: new_sink, closer: new_closer, resume_from, is_resume }) => {
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
                            closer = Some(new_closer);
                            detached_at = None;
                        }
                    }
                    Some(Ctl::Detach) => {
                        sink = None;
                        closer = None;
                        detached_at = Some(Instant::now());
                    }
                    Some(Ctl::Close) => {
                        // "unauthenticated" rather than a word of its own: it is
                        // what this token now is, and it is the one reason the
                        // client already answers by asking for a fresh token —
                        // which the server refuses, which signs them out. A
                        // reason nobody handles would leave them reconnecting
                        // into a locked door forever.
                        if let Some(tx) = &sink {
                            let frame = ServerFrame::control(ServerEvent::InvalidSession {
                                reason: "unauthenticated".into(),
                            });
                            if let Ok(json) = serde_json::to_string(&frame) {
                                let _ = tx.send(json).await;
                            }
                        }
                        // Queued behind that frame, so it goes out before the
                        // socket's writer runs dry and stops.
                        if let Some(closer) = closer.take() {
                            let _ = closer.send(());
                        }
                        break;
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
