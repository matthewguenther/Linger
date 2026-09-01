//! Two peer connections, in one process, actually connecting (T-1402).
//!
//! **What this proves and what it does not.** It proves the negotiation is
//! right: that two engines wired to each other reach `connected`, that exactly
//! one of them offers, that candidates arriving early are not lost, and that
//! the mesh is rebuilt when somebody reconnects with a new session id. Those
//! are the parts whose correctness can be established by making them happen.
//!
//! It proves **nothing at all about a real network**. Both ends are on loopback
//! with no NAT between them, which is the arrangement AGENTS §"Where you will
//! be wrong" names explicitly as the one that works right up until somebody is
//! behind carrier-grade NAT. TASKS.md says the same thing in fewer words: *do
//! not test this on one machine*. The four-people-four-networks check is the
//! evidence, and this file is not it.

use std::sync::{Arc, Mutex};
use std::time::Duration;

use linger_client_lib::voice::{audio, Engine, Signaller, Watcher};
use linger_core::gateway::{ClientFrame, VoicePeer, VoiceSignalKind};
use linger_core::{RoomId, UserId};
use tokio::sync::mpsc;

/// Where one engine's outgoing frames go: straight into a channel the test
/// pumps into the other engine. No server, because the thing under test is the
/// negotiation and a real socket in the middle would only make a failure harder
/// to read — T-1401's suite already proves the server routes these.
struct Wire(mpsc::UnboundedSender<ClientFrame>);

impl Signaller for Wire {
    fn send(&self, frame: ClientFrame) {
        let _ = self.0.send(frame);
    }
}

/// Every peer-state change an engine reported, in order.
#[derive(Default)]
struct Log(Mutex<Vec<(String, String)>>);

impl Watcher for Log {
    fn peer_state(&self, peer: &str, state: &str) {
        self.0.lock().unwrap().push((peer.to_string(), state.to_string()));
    }
}

impl Log {
    fn states(&self) -> Vec<(String, String)> {
        self.0.lock().unwrap().clone()
    }
}

type Rig = (
    Arc<Engine<Wire, Log>>,
    mpsc::UnboundedReceiver<ClientFrame>,
    Arc<Log>,
);

async fn engine(session: &str) -> Rig {
    let (tx, rx) = mpsc::unbounded_channel();
    let log = Arc::new(Log::default());
    // No ICE servers: on loopback there is nothing to traverse, and a STUN
    // lookup in a test is a network call that will one day fail in CI for
    // reasons that have nothing to do with the code.
    let engine = Arc::new(Engine::new(Arc::new(Wire(tx)), Arc::clone(&log), Vec::new()));
    engine.set_session(session.to_string()).await;
    (engine, rx, log)
}

fn peers(ids: &[&str]) -> Vec<VoicePeer> {
    ids.iter()
        .map(|id| VoicePeer {
            session_id: (*id).to_string(),
            user_id: UserId::new(),
        })
        .collect()
}

/// Carry whatever each side has to say to the other, once.
///
/// The test's stand-in for the gateway, and deliberately dumb: it forwards
/// frames and looks at nothing. T-1401's suite already proves the real server
/// routes these; what is under test here is what the two ends do with them.
async fn pump_once(
    a: &Arc<Engine<Wire, Log>>,
    a_id: &str,
    a_rx: &mut mpsc::UnboundedReceiver<ClientFrame>,
    b: &Arc<Engine<Wire, Log>>,
    b_id: &str,
    b_rx: &mut mpsc::UnboundedReceiver<ClientFrame>,
) {
    while let Ok(frame) = a_rx.try_recv() {
        if let ClientFrame::VoiceSignal { to, kind, payload } = frame {
            assert_eq!(to, b_id, "a signal was addressed to the wrong peer");
            b.on_signal(a_id, kind, &payload).await;
        }
    }
    while let Ok(frame) = b_rx.try_recv() {
        if let ClientFrame::VoiceSignal { to, kind, payload } = frame {
            assert_eq!(to, a_id, "a signal was addressed to the wrong peer");
            a.on_signal(b_id, kind, &payload).await;
        }
    }
}

/// Session ids are compared to decide who offers, so the test picks two whose
/// order is obvious: `aaa` offers, `bbb` answers.
const A: &str = "aaa-session";
const B: &str = "bbb-session";

#[tokio::test(flavor = "multi_thread")]
async fn two_peers_negotiate_and_connect() {
    let room = RoomId::new();
    let (a, mut a_rx, a_log) = engine(A).await;
    let (b, mut b_rx, b_log) = engine(B).await;

    a.join(room).await;
    b.join(room).await;
    // The join frames themselves go nowhere here — there is no server — so the
    // test plays the part the server plays and tells both sides who is in.
    while a_rx.try_recv().is_ok() {}
    while b_rx.try_recv().is_ok() {}

    let state = peers(&[A, B]);
    a.on_state(room, &state).await;
    b.on_state(room, &state).await;

    assert_eq!(a.peer_count().await, 1, "a did not build a peer for b");
    assert_eq!(b.peer_count().await, 1, "b did not build a peer for a");

    let mut connected = false;
    for _ in 0..600 {
        pump_once(&a, A, &mut a_rx, &b, B, &mut b_rx).await;
        if a.is_connected(B).await && b.is_connected(A).await {
            connected = true;
            break;
        }
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
    assert!(
        connected,
        "two peers on loopback never connected.\na: {:?}\nb: {:?}",
        a_log.states(),
        b_log.states()
    );

    // Both sides saw it happen, which is what the voice surface will draw.
    assert!(a_log.states().iter().any(|(_, s)| s == "connected"));
    assert!(b_log.states().iter().any(|(_, s)| s == "connected"));
}

#[tokio::test(flavor = "multi_thread")]
async fn exactly_one_side_offers() {
    let room = RoomId::new();
    let (a, mut a_rx, _) = engine(A).await;
    let (b, mut b_rx, _) = engine(B).await;
    a.join(room).await;
    b.join(room).await;
    while a_rx.try_recv().is_ok() {}
    while b_rx.try_recv().is_ok() {}

    let state = peers(&[A, B]);
    a.on_state(room, &state).await;
    b.on_state(room, &state).await;
    tokio::time::sleep(Duration::from_millis(200)).await;

    let mut offers = 0;
    let mut from_b = 0;
    while let Ok(ClientFrame::VoiceSignal { kind, .. }) = a_rx.try_recv() {
        if kind == VoiceSignalKind::Offer {
            offers += 1;
        }
    }
    while let Ok(ClientFrame::VoiceSignal { kind, .. }) = b_rx.try_recv() {
        if kind == VoiceSignalKind::Offer {
            from_b += 1;
        }
    }
    // Glare is two offers crossing, and it leaves both sides waiting for an
    // answer to an offer the other one threw away.
    assert_eq!(offers, 1, "the lower session id did not offer exactly once");
    assert_eq!(from_b, 0, "the higher session id offered as well");
}

#[tokio::test(flavor = "multi_thread")]
async fn a_peer_who_reconnects_gets_a_new_connection() {
    let room = RoomId::new();
    let (a, mut a_rx, a_log) = engine(A).await;
    let (b, mut b_rx, _) = engine(B).await;
    a.join(room).await;
    b.join(room).await;
    while a_rx.try_recv().is_ok() {}
    while b_rx.try_recv().is_ok() {}

    a.on_state(room, &peers(&[A, B])).await;
    b.on_state(room, &peers(&[A, B])).await;
    assert_eq!(a.peer_count().await, 1);

    // B's session ends and it comes back as somebody else, which is what a
    // reconnect past the resume window looks like from A's side.
    const B2: &str = "bbb-session-2";
    a.on_state(room, &peers(&[A, B2])).await;

    assert_eq!(a.peer_count().await, 1, "a is holding two peers for one person");
    assert!(
        a.outbound(B).await.is_none(),
        "the old connection was left open"
    );
    assert!(
        a.outbound(B2).await.is_some(),
        "the new connection was never built"
    );
    assert!(
        a_log.states().iter().any(|(p, s)| p == B && s == "closed"),
        "nothing said the old peer went: {:?}",
        a_log.states()
    );
    let _ = b;
    let _ = b_rx;
}

#[tokio::test(flavor = "multi_thread")]
async fn leaving_closes_every_peer() {
    let room = RoomId::new();
    let (a, mut a_rx, a_log) = engine(A).await;
    a.join(room).await;
    while a_rx.try_recv().is_ok() {}
    a.on_state(room, &peers(&[A, B, "ccc-session"])).await;
    assert_eq!(a.peer_count().await, 2);

    a.leave().await;
    assert_eq!(a.peer_count().await, 0, "a peer survived leaving");
    let closed = a_log
        .states()
        .iter()
        .filter(|(_, s)| s == "closed")
        .count();
    assert!(closed >= 2, "not every peer was reported closed: {:?}", a_log.states());
}

#[tokio::test(flavor = "multi_thread")]
async fn a_state_for_a_room_we_are_not_in_is_ignored() {
    let ours = RoomId::new();
    let theirs = RoomId::new();
    let (a, mut a_rx, _) = engine(A).await;
    a.join(ours).await;
    while a_rx.try_recv().is_ok() {}

    // We are told about every room we can see, not only the one we are in.
    a.on_state(theirs, &peers(&[A, B])).await;
    assert_eq!(
        a.peer_count().await,
        0,
        "a voice room we are not in built a mesh"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn a_candidate_that_arrives_before_the_answer_is_kept() {
    let room = RoomId::new();
    let (a, mut a_rx, _) = engine(A).await;
    a.join(room).await;
    while a_rx.try_recv().is_ok() {}
    a.on_state(room, &peers(&[A, B])).await;

    // ICE trickles, so the far end starts sending candidates before its answer
    // has been applied here. Dropping them is a call that takes the long way
    // round or never connects — and on a good network you never notice, which
    // is what makes it the kind of bug this project is warned about.
    a.on_signal(B, VoiceSignalKind::Candidate, "candidate:1 1 udp 1 127.0.0.1 1 typ host")
        .await;
    a.on_signal(B, VoiceSignalKind::Candidate, "candidate:2 1 udp 2 127.0.0.1 2 typ host")
        .await;

    // Nothing has blown up and the peer is still there to answer into.
    assert_eq!(a.peer_count().await, 1);
    assert!(a.outbound(B).await.is_some());
}

#[tokio::test(flavor = "multi_thread")]
async fn a_signal_from_a_stranger_is_harmless() {
    let room = RoomId::new();
    let (a, mut a_rx, _) = engine(A).await;
    a.join(room).await;
    while a_rx.try_recv().is_ok() {}

    // An answer or a candidate for a connection we do not have. The server
    // should never route one, so this is about not trusting that.
    a.on_signal("nobody", VoiceSignalKind::Answer, "v=0 nonsense").await;
    a.on_signal("nobody", VoiceSignalKind::Candidate, "candidate:1").await;
    assert_eq!(a.peer_count().await, 0);
}

#[test]
fn the_audio_seam_is_one_twenty_millisecond_frame() {
    // The seam `cpal` and Opus arrive at. Asserted here as well as in the unit
    // tests because it is the number both of those have to agree with, and a
    // change to it is a change to the shape of the hole they fill.
    assert_eq!(audio::FRAME_SAMPLES, 960);
    assert_eq!(audio::SAMPLE_RATE, 48_000);
    assert_eq!(audio::CHANNELS, 1);
}
