//! Voice: peer connections, driven by the gateway's signalling (SPEC §4.14,
//! ARCHITECTURE §2, T-1402).
//!
//! Audio lives in Rust rather than in the WebView, and this is where. The
//! server introduced us (T-1401); this module is what does something with the
//! introduction — one `RTCPeerConnection` per peer, a full mesh, and the
//! offer/answer/ICE dance on each of them.
//!
//! **The whole path is here now.** Real peer connections, real DTLS, real ICE,
//! real RTP — and, since the microphone half landed, real sound: `audio::Source`
//! frames are Opus-encoded once and written to every peer's outbound track,
//! and every inbound track is decoded and handed to the `audio::Sink`, which
//! mixes. The devices behind those two traits are `device.rs`; the tests use
//! the stand-ins in `audio.rs`.
//!
//! **Nothing in here has been across two networks.** AGENTS §"Where you will be
//! wrong" is explicit that WebRTC written from memory works on localhost and
//! dies behind carrier-grade NAT, and that warning applies to every line of
//! this file. What the tests prove is that two peers on one machine negotiate,
//! carry packets, and that a tone sent by one comes out of the other's sink;
//! what they cannot prove is anything about a real network, which is what
//! T-1402's acceptance criterion is for.

pub mod audio;
pub mod codec;
pub mod device;
pub mod mesh;

use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::Duration;

use bytes::Bytes;
use linger_core::gateway::{ClientFrame, VoicePeer, VoiceSignalKind};
use linger_core::RoomId;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use webrtc::api::interceptor_registry::register_default_interceptors;
use webrtc::api::media_engine::{MediaEngine, MIME_TYPE_OPUS};
use webrtc::api::APIBuilder;
use webrtc::ice_transport::ice_candidate::RTCIceCandidateInit;
use webrtc::ice_transport::ice_server::RTCIceServer;
use webrtc::interceptor::registry::Registry;
use webrtc::media::Sample;
use webrtc::peer_connection::configuration::RTCConfiguration;
use webrtc::peer_connection::sdp::session_description::RTCSessionDescription;
use webrtc::peer_connection::RTCPeerConnection;
use webrtc::rtp_transceiver::rtp_codec::RTCRtpCodecCapability;
use webrtc::track::track_local::track_local_static_sample::TrackLocalStaticSample;
use webrtc::track::track_remote::TrackRemote;

use audio::{Devices, Sink, Source};

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

    /// Our own audio changed state: `sending` when the microphone loop is
    /// running, `stopped` when it ended on its own — a device that went away
    /// — or a reason it could not start. Not a peer's business, so not
    /// `peer_state`.
    fn audio_state(&self, _state: &str) {}
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
    /// Shared with the sending loop and every inbound track's reader, which
    /// is why it is an `Arc` rather than a field.
    inner: Arc<Mutex<Inner>>,
}

#[derive(Default)]
struct Inner {
    /// Our own session id, once the gateway has told us. Everything about who
    /// offers depends on it, so nothing happens before it arrives.
    me: Option<String>,
    room: Option<RoomId>,
    peers: BTreeMap<String, Peer>,
    /// The microphone and the speakers, while we are in voice. Dropping them
    /// is what closes the devices.
    devices: Option<Devices>,
    /// The loop that carries microphone frames to every peer.
    pump: Option<JoinHandle<()>>,
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
            inner: Arc::new(Mutex::new(Inner::default())),
        }
    }

    /// Our own session id, from the gateway's `ready`.
    pub async fn set_session(&self, session_id: String) {
        self.inner.lock().await.me = Some(session_id);
    }

    /// Ask to join voice in a room, with the devices to do it through.
    ///
    /// The mesh is not built here — it is built when the server answers with
    /// a `voice.state` that has us in it, which is the same path a peer
    /// arriving later takes. What *does* start here is the sending loop: it
    /// encodes frames from the moment we join and writes them to whichever
    /// peers exist, so the first word after a connection comes up is not
    /// waiting on anything.
    pub async fn join(&self, room_id: RoomId, devices: Devices) {
        let source = Arc::clone(&devices.source);
        let previous = {
            let mut inner = self.inner.lock().await;
            inner.room = Some(room_id);
            inner.devices = Some(devices);
            inner.pump.take()
        };
        if let Some(previous) = previous {
            previous.abort();
        }
        let pump = tokio::spawn(pump(
            Arc::clone(&self.inner),
            source,
            Arc::clone(&self.watcher),
        ));
        self.inner.lock().await.pump = Some(pump);
        self.signaller.send(ClientFrame::VoiceJoin { room_id });
    }

    /// Leave, and tear the mesh down whether or not the server answers.
    ///
    /// The devices go too: leaving voice is the microphone turning off, and
    /// dropping the `Devices` is what closes it.
    pub async fn leave(&self) {
        self.signaller.send(ClientFrame::VoiceLeave);
        let (peers, devices, pump) = {
            let mut inner = self.inner.lock().await;
            inner.room = None;
            (
                std::mem::take(&mut inner.peers),
                inner.devices.take(),
                inner.pump.take(),
            )
        };
        if let Some(pump) = pump {
            // Abort, then wait for it to be gone: the loop holds the source,
            // and the microphone only closes once nobody does.
            pump.abort();
            let _ = pump.await;
        }
        for (id, peer) in peers {
            let _ = peer.conn.close().await;
            if let Some(devices) = &devices {
                devices.sink.forget(&id).await;
            }
            self.watcher.peer_state(&id, "closed");
        }
        // Closing the devices can block briefly while the audio threads are
        // joined, so it is done off the reactor.
        if let Some(devices) = devices {
            let _ = tokio::task::spawn_blocking(move || drop(devices)).await;
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
            let (peer, sink) = {
                let mut inner = self.inner.lock().await;
                (
                    inner.peers.remove(&id),
                    inner.devices.as_ref().map(|d| Arc::clone(&d.sink)),
                )
            };
            if let Some(peer) = peer {
                let _ = peer.conn.close().await;
                if let Some(sink) = sink {
                    sink.forget(&id).await;
                }
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
        // WebRTC audio is. The sending loop (`pump`) fills it.
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
        let sender = conn.add_track(Arc::clone(&outbound) as Arc<_>).await?;
        // The far end sends receiver reports and loss NACKs about our track,
        // and the interceptors only act on them if somebody reads them.
        // Nobody here needs the contents; the reading is the point.
        tokio::spawn(async move {
            let mut buffer = vec![0u8; 1500];
            while sender.read(&mut buffer).await.is_ok() {}
        });

        // Their audio, when it starts arriving. The sink is looked up when
        // the track appears rather than captured now, so a peer built before
        // devices exist (the tests do this) is not a peer with no ears.
        let inner = Arc::clone(&self.inner);
        let from = them.to_string();
        conn.on_track(Box::new(move |track, _receiver, _transceiver| {
            let inner = Arc::clone(&inner);
            let from = from.clone();
            Box::pin(async move {
                let sink = inner
                    .lock()
                    .await
                    .devices
                    .as_ref()
                    .map(|d| Arc::clone(&d.sink));
                let Some(sink) = sink else { return };
                tokio::spawn(receive(track, from, sink));
            })
        }));

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
                let Ok(init) = candidate.to_json() else {
                    return;
                };
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

    /// The track we send to one peer. For tests; the sending loop finds it
    /// on its own.
    pub async fn outbound(&self, peer: &str) -> Option<Arc<TrackLocalStaticSample>> {
        self.inner
            .lock()
            .await
            .peers
            .get(peer)
            .map(|p| Arc::clone(&p.outbound))
    }
}

/// The sending loop: microphone frames, encoded once, to every peer.
///
/// One encoder for the whole mesh rather than one per peer, because every
/// peer hears the same voice: encoding it eight times would be eight times
/// the CPU for identical bytes. The loop paces itself on the source — a
/// microphone delivers a frame every 20 ms, and so do the stand-ins.
///
/// It ends when the source does. That is a microphone that went away, and
/// until T-1405 makes it recover, the honest thing is to say so and stop.
async fn pump<W: Watcher>(inner: Arc<Mutex<Inner>>, source: Arc<dyn Source>, watcher: Arc<W>) {
    let mut encoder = match codec::Encoder::new() {
        Ok(encoder) => encoder,
        Err(error) => {
            watcher.audio_state(&format!("encoder: {error}"));
            return;
        }
    };
    watcher.audio_state("sending");
    while let Some(frame) = source.frame().await {
        let packet = match encoder.encode(&frame) {
            Ok(packet) => packet,
            Err(error) => {
                eprintln!("voice: encode: {error}");
                continue;
            }
        };
        let tracks: Vec<Arc<TrackLocalStaticSample>> = inner
            .lock()
            .await
            .peers
            .values()
            .map(|peer| Arc::clone(&peer.outbound))
            .collect();
        let sample = Sample {
            data: Bytes::from(packet),
            duration: Duration::from_millis(u64::from(audio::FRAME_MS)),
            ..Default::default()
        };
        for track in tracks {
            // A track whose connection is not up yet writes to nobody and
            // says so; that is the first second of every call, not an error.
            let _ = track.write_sample(&sample).await;
        }
    }
    watcher.audio_state("stopped");
}

/// The receiving loop for one peer's track: RTP in, frames to the sink.
///
/// An Opus RTP payload is one Opus packet (RFC 7587), so there is nothing to
/// reassemble. Sequence numbers are watched for the one thing worth doing
/// about a gap: asking the decoder to conceal each missing frame, so a lost
/// packet is a smear rather than a click, and the far end's clock keeps its
/// place. A gap of more than a few is a pause, not loss, and is left alone.
async fn receive(track: Arc<TrackRemote>, peer: String, sink: Arc<dyn Sink>) {
    let mut decoder = match codec::Decoder::new() {
        Ok(decoder) => decoder,
        Err(error) => {
            eprintln!("voice: peer {peer}: decoder: {error}");
            return;
        }
    };
    let mut expected: Option<u16> = None;
    while let Ok((packet, _)) = track.read_rtp().await {
        let sequence = packet.header.sequence_number;
        if let Some(expected) = expected {
            let gap = sequence.wrapping_sub(expected);
            if (1..5).contains(&gap) {
                for _ in 0..gap {
                    if let Ok(guess) = decoder.conceal() {
                        sink.play(&peer, &guess).await;
                    }
                }
            }
        }
        expected = Some(sequence.wrapping_add(1));
        if packet.payload.is_empty() {
            continue;
        }
        match decoder.decode(&packet.payload) {
            Ok(samples) => sink.play(&peer, &samples).await,
            Err(error) => eprintln!("voice: peer {peer}: decode: {error}"),
        }
    }
}

/// Somewhere for an error to go that is not a panic and not silence.
///
/// A dropped signal is not fatal — the next `voice.state` rebuilds — so a
/// failure here has to be visible without taking the call down with it.
fn tracing_error(peer: &str, error: &webrtc::Error) {
    eprintln!("voice: peer {peer}: {error}");
}
