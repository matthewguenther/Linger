//! Voice: peer connections, driven by the gateway's signalling (SPEC §4.14,
//! ARCHITECTURE §2, T-1402).
//!
//! Audio lives in Rust rather than in the WebView, and this is where. The
//! server introduced us (T-1401); this module is what does something with the
//! introduction — one `RTCPeerConnection` per peer, a full mesh, and the
//! offer/answer/ICE dance on each of them.
//!
//! **What is here and what is not.** The transport is real: real peer
//! connections, real DTLS, real ICE, real RTP. The microphone and the speakers
//! are not — `cpal` needs ALSA's headers on Linux and an Opus encoder needs
//! libopus, and neither is installable without a password. `audio::Source` and
//! `audio::Sink` are the seam they arrive at, and the seam is one 20 ms frame,
//! which is what both sides of it want anyway.
//!
//! **Nothing in here has been across two networks.** AGENTS §"Where you will be
//! wrong" is explicit that WebRTC written from memory works on localhost and
//! dies behind carrier-grade NAT, and that warning applies to every line of
//! this file. What the tests prove is that two peers on one machine negotiate
//! and carry packets; what they cannot prove is anything about a real network,
//! which is what T-1402's acceptance criterion is for.

pub mod audio;
pub mod mesh;

use std::collections::BTreeMap;
use std::sync::Arc;

use linger_core::gateway::{ClientFrame, VoicePeer, VoiceSignalKind};
use linger_core::RoomId;
use tokio::sync::Mutex;
use webrtc::api::interceptor_registry::register_default_interceptors;
use webrtc::api::media_engine::{MediaEngine, MIME_TYPE_OPUS};
use webrtc::api::APIBuilder;
use webrtc::ice_transport::ice_candidate::RTCIceCandidateInit;
use webrtc::ice_transport::ice_server::RTCIceServer;
use webrtc::interceptor::registry::Registry;
use webrtc::peer_connection::configuration::RTCConfiguration;
use webrtc::peer_connection::sdp::session_description::RTCSessionDescription;
use webrtc::peer_connection::RTCPeerConnection;
use webrtc::rtp_transceiver::rtp_codec::RTCRtpCodecCapability;
use webrtc::track::track_local::track_local_static_sample::TrackLocalStaticSample;

/// Where a signal goes when this module wants to send one.
///
/// A trait rather than a channel to the gateway, so the tests can wire two
/// engines straight to each other and leave the server out of it — the thing
/// being tested is the negotiation, and a real socket in the middle would only
/// make a failure harder to read.
pub trait Signaller: Send + Sync + 'static {
    fn send(&self, frame: ClientFrame);
}

/// What the engine tells the app about.
///
/// Deliberately small: T-1404 draws the voice surface and will want more, but
/// inventing what it wants before it exists is how a callback ends up carrying
/// three fields nobody reads.
pub trait Watcher: Send + Sync + 'static {
    /// A peer's connection changed state — `connected`, `failed`, `closed`.
    fn peer_state(&self, peer: &str, state: &str);
}

/// One remote client, and the connection to it.
struct Peer {
    conn: Arc<RTCPeerConnection>,
    /// What we are sending them. Held so the sending loop can find it.
    outbound: Arc<TrackLocalStaticSample>,
    /// Candidates that turned up before the answer did.
    ///
    /// ICE trickles: the other end starts sending candidates as soon as it has
    /// them, which is routinely before its answer has been applied here. Adding
    /// one to a connection with no remote description is an error, and dropping
    /// it is a connection that takes the long way round or never connects at
    /// all — on a good network you never notice, which is what makes it the
    /// kind of bug this project is warned about.
    pending: Vec<RTCIceCandidateInit>,
}

/// The mesh, and everything it is doing.
pub struct Engine<S: Signaller, W: Watcher> {
    signaller: Arc<S>,
    watcher: Arc<W>,
    ice_servers: Vec<RTCIceServer>,
    inner: Mutex<Inner>,
}

#[derive(Default)]
struct Inner {
    /// Our own session id, once the gateway has told us. Everything about who
    /// offers depends on it, so nothing happens before it arrives.
    me: Option<String>,
    room: Option<RoomId>,
    peers: BTreeMap<String, Peer>,
}

impl<S: Signaller, W: Watcher> Engine<S, W> {
    /// `ice_servers` is STUN and, later, TURN (T-1403). Empty means host
    /// candidates only, which is enough for two machines on one network and
    /// nothing else — that is the whole reason T-1403 exists.
    #[must_use]
    pub fn new(signaller: Arc<S>, watcher: Arc<W>, ice_servers: Vec<RTCIceServer>) -> Self {
        Self {
            signaller,
            watcher,
            ice_servers,
            inner: Mutex::new(Inner::default()),
        }
    }

    /// Our own session id, from the gateway's `ready`.
    pub async fn set_session(&self, session_id: String) {
        self.inner.lock().await.me = Some(session_id);
    }

    /// Ask to join voice in a room. The mesh is not built here — it is built
    /// when the server answers with a `voice.state` that has us in it, which is
    /// the same path a peer arriving later takes.
    pub async fn join(&self, room_id: RoomId) {
        self.inner.lock().await.room = Some(room_id);
        self.signaller.send(ClientFrame::VoiceJoin { room_id });
    }

    /// Leave, and tear the mesh down whether or not the server answers.
    pub async fn leave(&self) {
        self.signaller.send(ClientFrame::VoiceLeave);
        let peers = {
            let mut inner = self.inner.lock().await;
            inner.room = None;
            std::mem::take(&mut inner.peers)
        };
        for (id, peer) in peers {
            let _ = peer.conn.close().await;
            self.watcher.peer_state(&id, "closed");
        }
    }

    /// The server's picture of who is in voice. Reconcile ours with it.
    pub async fn on_state(&self, room_id: RoomId, peers: &[VoicePeer]) {
        let (me, plan) = {
            let inner = self.inner.lock().await;
            // A state for a room we are not in is somebody else's business —
            // we are told about every room we can see, not only ours.
            if inner.room != Some(room_id) {
                return;
            }
            let Some(me) = inner.me.clone() else { return };
            let held = inner.peers.keys().cloned().collect();
            let plan = mesh::plan(&me, &held, peers);
            (me, plan)
        };

        for id in plan.drop {
            let peer = self.inner.lock().await.peers.remove(&id);
            if let Some(peer) = peer {
                let _ = peer.conn.close().await;
                self.watcher.peer_state(&id, "closed");
            }
        }
        for id in plan.connect {
            if let Err(error) = self.open(&me, &id).await {
                // A peer that failed to build is not fatal to the others: a
                // mesh with a hole in it is still a call, and the next
                // `voice.state` tries again.
                self.watcher.peer_state(&id, "failed");
                tracing_error(&id, &error);
            }
        }
    }

    /// A signal from one peer.
    pub async fn on_signal(&self, from: &str, kind: VoiceSignalKind, payload: &str) {
        if let Err(error) = self.apply_signal(from, kind, payload).await {
            tracing_error(from, &error);
        }
    }

    async fn apply_signal(
        &self,
        from: &str,
        kind: VoiceSignalKind,
        payload: &str,
    ) -> Result<(), webrtc::Error> {
        match kind {
            VoiceSignalKind::Offer => {
                // An offer from somebody we have no connection to is the
                // ordinary case for the side that does not offer: they told us
                // first and we build on hearing from them.
                let me = {
                    let inner = self.inner.lock().await;
                    let Some(me) = inner.me.clone() else {
                        return Ok(());
                    };
                    me
                };
                if !self.inner.lock().await.peers.contains_key(from) {
                    self.open(&me, from).await?;
                }
                let sdp = RTCSessionDescription::offer(payload.to_string())?;
                let conn = self.conn_of(from).await;
                let Some(conn) = conn else { return Ok(()) };
                conn.set_remote_description(sdp).await?;
                let answer = conn.create_answer(None).await?;
                conn.set_local_description(answer.clone()).await?;
                self.signaller.send(ClientFrame::VoiceSignal {
                    to: from.to_string(),
                    kind: VoiceSignalKind::Answer,
                    payload: answer.sdp,
                });
                self.drain_pending(from).await?;
            }
            VoiceSignalKind::Answer => {
                let Some(conn) = self.conn_of(from).await else {
                    return Ok(());
                };
                let sdp = RTCSessionDescription::answer(payload.to_string())?;
                conn.set_remote_description(sdp).await?;
                self.drain_pending(from).await?;
            }
            VoiceSignalKind::Candidate => {
                let candidate = RTCIceCandidateInit {
                    candidate: payload.to_string(),
                    ..Default::default()
                };
                let Some(conn) = self.conn_of(from).await else {
                    return Ok(());
                };
                // Before the remote description lands, a candidate has nothing
                // to attach to. Held rather than dropped — see `Peer::pending`.
                if conn.remote_description().await.is_none() {
                    if let Some(peer) = self.inner.lock().await.peers.get_mut(from) {
                        peer.pending.push(candidate);
                    }
                    return Ok(());
                }
                conn.add_ice_candidate(candidate).await?;
            }
        }
        Ok(())
    }

    async fn conn_of(&self, peer: &str) -> Option<Arc<RTCPeerConnection>> {
        self.inner
            .lock()
            .await
            .peers
            .get(peer)
            .map(|p| Arc::clone(&p.conn))
    }

    /// Apply the candidates that arrived before the description they belong to.
    async fn drain_pending(&self, peer: &str) -> Result<(), webrtc::Error> {
        let (conn, pending) = {
            let mut inner = self.inner.lock().await;
            let Some(held) = inner.peers.get_mut(peer) else {
                return Ok(());
            };
            (Arc::clone(&held.conn), std::mem::take(&mut held.pending))
        };
        for candidate in pending {
            conn.add_ice_candidate(candidate).await?;
        }
        Ok(())
    }

    /// Build one peer connection, and offer if we are the one who offers.
    async fn open(&self, me: &str, them: &str) -> Result<(), webrtc::Error> {
        let mut media = MediaEngine::default();
        media.register_default_codecs()?;
        let mut registry = Registry::new();
        registry = register_default_interceptors(registry, &mut media)?;
        let api = APIBuilder::new()
            .with_media_engine(media)
            .with_interceptor_registry(registry)
            .build();

        let conn = Arc::new(
            api.new_peer_connection(RTCConfiguration {
                ice_servers: self.ice_servers.clone(),
                ..Default::default()
            })
            .await?,
        );

        // One outbound audio track, negotiated as Opus because that is what
        // WebRTC audio is. What fills it is `audio::Source`, and the encoder
        // that turns frames into Opus is the piece that is not here yet.
        let outbound = Arc::new(TrackLocalStaticSample::new(
            RTCRtpCodecCapability {
                mime_type: MIME_TYPE_OPUS.to_owned(),
                clock_rate: audio::SAMPLE_RATE,
                channels: audio::CHANNELS,
                ..Default::default()
            },
            "audio".to_owned(),
            format!("linger-{me}"),
        ));
        conn.add_track(Arc::clone(&outbound) as Arc<_>).await?;

        // Trickle: send each candidate as it is found rather than waiting for
        // gathering to finish. Waiting is a second or more of silence at the
        // start of every call, and on a bad network it is much worse.
        let signaller = Arc::clone(&self.signaller);
        let to = them.to_string();
        conn.on_ice_candidate(Box::new(move |candidate| {
            let signaller = Arc::clone(&signaller);
            let to = to.clone();
            Box::pin(async move {
                let Some(candidate) = candidate else { return };
                let Ok(init) = candidate.to_json() else { return };
                signaller.send(ClientFrame::VoiceSignal {
                    to,
                    kind: VoiceSignalKind::Candidate,
                    payload: init.candidate,
                });
            })
        }));

        let watcher = Arc::clone(&self.watcher);
        let who = them.to_string();
        conn.on_peer_connection_state_change(Box::new(move |state| {
            watcher.peer_state(&who, &state.to_string());
            Box::pin(async {})
        }));

        self.inner.lock().await.peers.insert(
            them.to_string(),
            Peer {
                conn: Arc::clone(&conn),
                outbound,
                pending: Vec::new(),
            },
        );

        if mesh::we_offer(me, them) {
            let offer = conn.create_offer(None).await?;
            conn.set_local_description(offer.clone()).await?;
            self.signaller.send(ClientFrame::VoiceSignal {
                to: them.to_string(),
                kind: VoiceSignalKind::Offer,
                payload: offer.sdp,
            });
        }
        Ok(())
    }

    /// How many peers we hold. For tests and for the surface T-1404 will draw.
    pub async fn peer_count(&self) -> usize {
        self.inner.lock().await.peers.len()
    }

    /// Whether a peer's connection has reached `connected`.
    pub async fn is_connected(&self, peer: &str) -> bool {
        let Some(conn) = self.conn_of(peer).await else {
            return false;
        };
        conn.connection_state()
            == webrtc::peer_connection::peer_connection_state::RTCPeerConnectionState::Connected
    }

    /// The track we send to one peer, for the loop that will fill it.
    pub async fn outbound(&self, peer: &str) -> Option<Arc<TrackLocalStaticSample>> {
        self.inner
            .lock()
            .await
            .peers
            .get(peer)
            .map(|p| Arc::clone(&p.outbound))
    }
}

/// Somewhere for an error to go that is not a panic and not silence.
///
/// A dropped signal is not fatal — the next `voice.state` rebuilds — so a
/// failure here has to be visible without taking the call down with it.
fn tracing_error(peer: &str, error: &webrtc::Error) {
    eprintln!("voice: peer {peer}: {error}");
}
