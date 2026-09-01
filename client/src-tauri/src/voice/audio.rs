//! The two ends of the audio path: where sound comes from, and where it goes
//! (SPEC §4.14, ARCHITECTURE §2, T-1402).
//!
//! **The microphone and the speakers are not here yet, and this file is the
//! shape of the hole they go in.** `cpal` needs ALSA's development headers on
//! Linux and an Opus encoder needs libopus; neither is installable without a
//! password, so what exists today is the seam plus a source that produces
//! silence. Everything on the far side of the seam — peer connections, ICE,
//! RTP, the mesh — is real and runs.
//!
//! The seam is one 20 ms frame in each direction, which is not an invented
//! boundary: it is the frame size Opus and WebRTC both work in, so `cpal` fills
//! the same buffer this hands out and there is nothing to redesign when it
//! arrives.
//!
//! Two things are deliberately *not* abstracted here, because guessing at them
//! is how a seam ends up in the wrong place:
//!
//! - **Device selection** is T-1405's, along with hotplug and the OS default
//!   changing under a call. A picker on top of a device list nobody has
//!   enumerated would be a guess about an API this crate has not linked yet.
//! - **Mixing several peers** happens on the playback side, and where it
//!   belongs depends on whether `cpal` gives us one output stream or one per
//!   device. [`Sink::play`] takes the peer it came from so that decision stays
//!   open.

use async_trait::async_trait;

/// The sample rate everything in here runs at.
///
/// 48 kHz because that is what Opus encodes and what WebRTC negotiates; a
/// device that wants something else is resampled at the edge rather than
/// halfway through, so nothing between here and the network has to think about
/// it. (The resampling is T-1405's problem, and it is on that list because a
/// sample-rate mismatch is one of the three ways audio devices break.)
pub const SAMPLE_RATE: u32 = 48_000;

/// One channel. Voice is mono — a second channel doubles the bytes on a mesh
/// and carries nothing anybody can hear on a laptop microphone.
pub const CHANNELS: u16 = 1;

/// How much audio is in one frame, in milliseconds.
///
/// Twenty is Opus's default and WebRTC's usual: ten doubles the packet rate for
/// latency nobody notices, and forty saves nothing worth the delay it adds.
pub const FRAME_MS: u32 = 20;

/// Samples in one frame. 960 at 48 kHz.
pub const FRAME_SAMPLES: usize = (SAMPLE_RATE as usize / 1000) * FRAME_MS as usize;

/// Where the sound going out comes from.
///
/// One frame per call, already at [`SAMPLE_RATE`] and [`CHANNELS`]. Returning
/// `None` means "nothing to send right now", which is not the same as silence:
/// a muted microphone should send silence so the far end's jitter buffer keeps
/// running, and a source that has stopped should send nothing at all.
#[async_trait]
pub trait Source: Send + Sync + 'static {
    async fn frame(&self) -> Option<Vec<i16>>;
}

/// Where the sound coming in goes.
///
/// `peer` is the session it arrived from, because playback has to mix several
/// people and cannot do that without knowing who is who.
#[async_trait]
pub trait Sink: Send + Sync + 'static {
    async fn play(&self, peer: &str, samples: &[i16]);
}

/// A microphone that is not there.
///
/// It produces real frames of silence at the real rate, so the whole path —
/// packetise, encrypt, send, receive, decrypt, hand to the sink — runs and can
/// be measured. What it cannot prove is that anybody can hear anything, which
/// is why T-1402's acceptance criterion is four people and not a test.
pub struct Silence;

#[async_trait]
impl Source for Silence {
    async fn frame(&self) -> Option<Vec<i16>> {
        tokio::time::sleep(std::time::Duration::from_millis(u64::from(FRAME_MS))).await;
        Some(vec![0i16; FRAME_SAMPLES])
    }
}

/// A tone, for proving something arrived.
///
/// A sink cannot tell silence from a dead connection, so the tests send this
/// instead: it is a 440 Hz sine, and a frame of it that comes out the other end
/// is a frame that really crossed a peer connection.
pub struct Tone {
    at: std::sync::atomic::AtomicU64,
}

impl Default for Tone {
    fn default() -> Self {
        Self {
            at: std::sync::atomic::AtomicU64::new(0),
        }
    }
}

#[async_trait]
impl Source for Tone {
    async fn frame(&self) -> Option<Vec<i16>> {
        use std::sync::atomic::Ordering;
        tokio::time::sleep(std::time::Duration::from_millis(u64::from(FRAME_MS))).await;
        let start = self.at.fetch_add(FRAME_SAMPLES as u64, Ordering::Relaxed);
        Some(
            (0..FRAME_SAMPLES)
                .map(|n| {
                    #[allow(clippy::cast_precision_loss)]
                    let t = (start + n as u64) as f32 / SAMPLE_RATE as f32;
                    #[allow(clippy::cast_possible_truncation)]
                    let value = (t * 440.0 * std::f32::consts::TAU).sin() * 8000.0;
                    value as i16
                })
                .collect(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_frame_is_twenty_milliseconds_at_forty_eight_kilohertz() {
        assert_eq!(FRAME_SAMPLES, 960);
    }

    #[tokio::test]
    async fn silence_is_a_real_frame_of_nothing() {
        let frame = Silence.frame().await.expect("a frame");
        assert_eq!(frame.len(), FRAME_SAMPLES);
        assert!(frame.iter().all(|s| *s == 0));
    }

    /// The tone has to be something a sink can recognise, or it is no better
    /// than silence for proving a frame arrived.
    #[tokio::test]
    async fn the_tone_is_audible_and_keeps_going() {
        let tone = Tone::default();
        let first = tone.frame().await.expect("a frame");
        let second = tone.frame().await.expect("a frame");
        assert_eq!(first.len(), FRAME_SAMPLES);
        assert!(
            first.iter().any(|s| s.abs() > 1000),
            "the tone is inaudible"
        );
        assert_ne!(
            first, second,
            "the tone repeated, so its phase is not advancing"
        );
    }
}
