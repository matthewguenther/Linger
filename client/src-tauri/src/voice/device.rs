//! The microphone and the speakers, through `cpal` (SPEC §4.14, ARCHITECTURE
//! §2, T-1402).
//!
//! This is the file `audio.rs` said would arrive: the two ends of the seam,
//! filled. [`Microphone`] is an `audio::Source` that produces one 20 ms frame
//! at a time from the default input device, and [`Speaker`] is an
//! `audio::Sink` that mixes every peer's frames into the default output
//! device. Nothing else in the voice path knows a device exists.
//!
//! **Two things live here that AGENTS §"Where you will be wrong" names, and
//! both are handled in the smallest way that is not wrong:**
//!
//! - **Sample rate.** Everything past this file is 48 kHz mono. A device that
//!   will not do 48 kHz gets its native rate and a linear resampler at the
//!   edge — good enough for a voice, and confined to one struct so T-1405 can
//!   replace it without touching anything else.
//! - **Hotplug and the default device changing** are *not* handled. A device
//!   that disappears ends the stream: the microphone reports it and the
//!   engine's sending loop stops. That is T-1405's job, and it is its own task
//!   because getting it right is most of the work.
//!
//! **Threads.** A `cpal` stream is driven by a thread the library owns, and
//! the stream handle itself is not something to hand between threads. So each
//! device gets one worker thread of ours that opens the device, starts the
//! stream, and then sleeps until told to stop — dropping the stream on the
//! same thread that built it. The callbacks talk to the rest of the world
//! through channels and one mutex, and never block.

use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{SampleFormat, SupportedStreamConfig, SupportedStreamConfigRange};
use tokio::sync::mpsc;

use crate::voice::audio::{Devices, Sink, Source, CHANNELS, FRAME_SAMPLES, SAMPLE_RATE};

/// Why a device could not be opened, in words the voice surface can show.
#[derive(Debug, thiserror::Error)]
pub enum DeviceError {
    #[error("no {0} device")]
    NoDevice(&'static str),
    #[error("the {0} device produces {1} samples, which this build cannot read")]
    Format(&'static str, SampleFormat),
    #[error("{0}")]
    Cpal(#[from] cpal::Error),
    #[error("could not start the audio thread: {0}")]
    Thread(#[from] std::io::Error),
    #[error("the audio thread stopped before the device was ready")]
    Gone,
}

/// Open the default microphone and the default speakers.
///
/// Both or neither: joining voice with no way to hear anybody is not a
/// conversation, and joining with no microphone is what mute is for.
pub fn open_default() -> Result<Devices, DeviceError> {
    let source = Microphone::open()?;
    let sink = Speaker::open()?;
    Ok(Devices {
        source: Arc::new(source),
        sink: Arc::new(sink),
    })
}

/// The frames a microphone produces, and a way to stop it.
pub struct Microphone {
    frames: tokio::sync::Mutex<mpsc::Receiver<Vec<i16>>>,
    _worker: Worker,
}

impl Microphone {
    /// Open the default input device.
    ///
    /// Blocks while the device is opened, which on some hosts is tens of
    /// milliseconds — call it from a blocking task, not from the reactor.
    pub fn open() -> Result<Self, DeviceError> {
        // Sixteen frames is a third of a second. The callback drops frames if
        // the engine falls further behind than that, because a queue that
        // grows is latency nobody asked for.
        let (tx, rx) = mpsc::channel::<Vec<i16>>(16);
        let worker = Worker::start(move || {
            let device = cpal::default_host()
                .default_input_device()
                .ok_or(DeviceError::NoDevice("input"))?;
            let config = match pick(device.supported_input_configs()?) {
                Some(config) => config,
                None => device.default_input_config()?,
            };
            let mut framer = Framer::new(config.channels(), config.sample_rate(), tx.clone());
            let died = move |error: cpal::Error| {
                eprintln!("voice: microphone: {error}");
                // An empty frame is the sentinel `frame()` reads as "the
                // device is gone". `blocking_send` is fine: this is cpal's
                // thread, not the reactor's.
                let _ = tx.blocking_send(Vec::new());
            };
            let stream = match config.sample_format() {
                SampleFormat::I16 => device.build_input_stream(
                    config.config(),
                    move |data: &[i16], _| framer.push(data.iter().copied()),
                    died,
                    None,
                )?,
                SampleFormat::F32 => device.build_input_stream(
                    config.config(),
                    move |data: &[f32], _| framer.push(data.iter().map(|s| from_f32(*s))),
                    died,
                    None,
                )?,
                other => return Err(DeviceError::Format("input", other)),
            };
            Ok((stream, ()))
        })?;
        Ok(Self {
            frames: tokio::sync::Mutex::new(rx),
            _worker: worker,
        })
    }
}

#[async_trait]
impl Source for Microphone {
    async fn frame(&self) -> Option<Vec<i16>> {
        let frame = self.frames.lock().await.recv().await?;
        // The sentinel from the error callback: the device is gone, and so is
        // this source. Returning `None` is what ends the engine's sending
        // loop, which is the honest outcome until T-1405 makes it recover.
        if frame.is_empty() {
            return None;
        }
        Some(frame)
    }
}

/// Mixes every peer into the default output device.
pub struct Speaker {
    lanes: Arc<Mutex<HashMap<String, Lane>>>,
    /// The device's rate. Frames arrive at `SAMPLE_RATE` and are resampled on
    /// the way into a lane if this differs, so the callback only ever copies.
    rate: u32,
    _worker: Worker<u32>,
}

/// One peer's audio, waiting to be played.
struct Lane {
    queue: VecDeque<i16>,
    resampler: Option<Linear>,
}

impl Speaker {
    /// Open the default output device. Blocks like [`Microphone::open`].
    pub fn open() -> Result<Self, DeviceError> {
        let lanes: Arc<Mutex<HashMap<String, Lane>>> = Arc::new(Mutex::new(HashMap::new()));
        let shared = Arc::clone(&lanes);
        let worker = Worker::start(move || {
            let device = cpal::default_host()
                .default_output_device()
                .ok_or(DeviceError::NoDevice("output"))?;
            let config = match pick(device.supported_output_configs()?) {
                Some(config) => config,
                None => device.default_output_config()?,
            };
            let channels = usize::from(config.channels());
            let rate = config.sample_rate();
            let died = |error: cpal::Error| eprintln!("voice: speaker: {error}");
            let stream = match config.sample_format() {
                SampleFormat::I16 => {
                    let lanes = Arc::clone(&shared);
                    device.build_output_stream(
                        config.config(),
                        move |out: &mut [i16], _| mix(&lanes, channels, out, |s| s),
                        died,
                        None,
                    )?
                }
                SampleFormat::F32 => {
                    let lanes = Arc::clone(&shared);
                    device.build_output_stream(
                        config.config(),
                        move |out: &mut [f32], _| mix(&lanes, channels, out, to_f32),
                        died,
                        None,
                    )?
                }
                other => return Err(DeviceError::Format("output", other)),
            };
            Ok((stream, rate))
        })?;
        Ok(Self {
            lanes,
            rate: worker.info,
            _worker: worker,
        })
    }
}

/// How far behind playback is allowed to fall before old audio is thrown
/// away: 200 ms. Past that, a queue is not absorbing jitter, it is adding
/// delay to every word from here on.
const MAX_QUEUED_MS: u32 = 200;

#[async_trait]
impl Sink for Speaker {
    async fn play(&self, peer: &str, samples: &[i16]) {
        let cap = (self.rate / 1000 * MAX_QUEUED_MS) as usize;
        let mut lanes = lock(&self.lanes);
        let lane = lanes.entry(peer.to_string()).or_insert_with(|| Lane {
            queue: VecDeque::new(),
            resampler: (self.rate != SAMPLE_RATE).then(|| Linear::new(SAMPLE_RATE, self.rate)),
        });
        match lane.resampler.as_mut() {
            Some(resampler) => {
                let mut out = Vec::with_capacity(samples.len());
                resampler.push(samples, &mut out);
                lane.queue.extend(out);
            }
            None => lane.queue.extend(samples),
        }
        while lane.queue.len() > cap {
            lane.queue.pop_front();
        }
    }

    async fn forget(&self, peer: &str) {
        lock(&self.lanes).remove(peer);
    }
}

/// The output callback: one sample from every lane, summed, into every
/// channel of the device. A lane with nothing queued contributes silence,
/// which is what a pause between words is.
fn mix<T: Copy>(
    lanes: &Mutex<HashMap<String, Lane>>,
    channels: usize,
    out: &mut [T],
    convert: impl Fn(i16) -> T,
) {
    let mut lanes = lock(lanes);
    for frame in out.chunks_mut(channels.max(1)) {
        let mut acc: i32 = 0;
        for lane in lanes.values_mut() {
            if let Some(sample) = lane.queue.pop_front() {
                acc += i32::from(sample);
            }
        }
        #[allow(clippy::cast_possible_truncation)]
        let sample = acc.clamp(i32::from(i16::MIN), i32::from(i16::MAX)) as i16;
        for slot in frame {
            *slot = convert(sample);
        }
    }
}

/// A mutex that survives a panic on the other side. Audio callbacks must not
/// be the thing that stops the app.
fn lock<T>(mutex: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

/// Prefer 48 kHz, prefer mono, and take either sample format the callbacks
/// read. `None` means the device offers no 48 kHz at all and the caller
/// should take the default and resample.
fn pick(ranges: impl Iterator<Item = SupportedStreamConfigRange>) -> Option<SupportedStreamConfig> {
    let mut best: Option<SupportedStreamConfig> = None;
    for range in ranges {
        if !matches!(range.sample_format(), SampleFormat::I16 | SampleFormat::F32) {
            continue;
        }
        let Some(config) = range.try_with_sample_rate(SAMPLE_RATE) else {
            continue;
        };
        if best.as_ref().is_none_or(|held| rank(&config) < rank(held)) {
            best = Some(config);
        }
    }
    best
}

/// Lower is better: fewest channels beyond one, then i16 over f32 because it
/// is what the rest of the path speaks.
fn rank(config: &SupportedStreamConfig) -> (u16, u8) {
    let channels = config.channels().saturating_sub(CHANNELS);
    let format = u8::from(config.sample_format() != SampleFormat::I16);
    (channels, format)
}

#[allow(clippy::cast_possible_truncation)]
fn from_f32(sample: f32) -> i16 {
    (sample.clamp(-1.0, 1.0) * f32::from(i16::MAX)) as i16
}

fn to_f32(sample: i16) -> f32 {
    f32::from(sample) / -f32::from(i16::MIN)
}

/// Turns whatever the device delivers into frames of mono at `SAMPLE_RATE`.
struct Framer {
    channels: usize,
    resampler: Option<Linear>,
    pending: Vec<i16>,
    tx: mpsc::Sender<Vec<i16>>,
}

impl Framer {
    fn new(channels: u16, rate: u32, tx: mpsc::Sender<Vec<i16>>) -> Self {
        Self {
            channels: usize::from(channels).max(1),
            resampler: (rate != SAMPLE_RATE).then(|| Linear::new(rate, SAMPLE_RATE)),
            pending: Vec::with_capacity(FRAME_SAMPLES * 2),
            tx,
        }
    }

    /// Interleaved samples in; whole frames out, as many as are ready.
    fn push(&mut self, samples: impl Iterator<Item = i16>) {
        let mono = downmix(samples, self.channels);
        match self.resampler.as_mut() {
            Some(resampler) => resampler.push(&mono, &mut self.pending),
            None => self.pending.extend(mono),
        }
        while self.pending.len() >= FRAME_SAMPLES {
            let frame: Vec<i16> = self.pending.drain(..FRAME_SAMPLES).collect();
            // Dropped rather than queued if the engine is behind — see the
            // channel size in `Microphone::open`.
            let _ = self.tx.try_send(frame);
        }
    }
}

/// Average the channels of each interleaved frame into one sample.
fn downmix(samples: impl Iterator<Item = i16>, channels: usize) -> Vec<i16> {
    let mut out = Vec::new();
    let mut acc: i32 = 0;
    let mut n = 0;
    for sample in samples {
        acc += i32::from(sample);
        n += 1;
        if n == channels {
            #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
            out.push((acc / channels as i32) as i16);
            acc = 0;
            n = 0;
        }
    }
    out
}

/// Linear interpolation between two rates.
///
/// The simplest resampler that is not wrong for a voice: it aliases a little
/// above what anybody says, and it costs one multiply per sample. It keeps
/// its place across calls, so frames of any length go in and nothing is
/// dropped at the boundaries.
struct Linear {
    /// Input samples per output sample.
    step: f64,
    /// Where the next output sample falls, in input samples, measured from
    /// the sample before this call's input (`last`).
    pos: f64,
    last: i16,
}

impl Linear {
    fn new(from: u32, to: u32) -> Self {
        Self {
            step: f64::from(from) / f64::from(to),
            pos: 0.0,
            last: 0,
        }
    }

    fn push(&mut self, input: &[i16], out: &mut Vec<i16>) {
        let Some(newest) = input.last() else { return };
        // Position 0 is the sample carried from the previous call, position k
        // is `input[k - 1]`; output lands between two of those.
        let at = |k: usize| {
            if k == 0 {
                f64::from(self.last)
            } else {
                f64::from(input[k - 1])
            }
        };
        let end = input.len() as f64;
        let mut pos = self.pos;
        while pos < end {
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let k = pos as usize;
            let frac = pos - k as f64;
            let a = at(k);
            let b = at(k + 1);
            #[allow(clippy::cast_possible_truncation)]
            out.push((a + (b - a) * frac).round() as i16);
            pos += self.step;
        }
        self.pos = pos - end;
        self.last = *newest;
    }
}

/// A thread that owns one `cpal` stream for as long as the device is wanted.
///
/// Not joined on drop, on purpose. The last handle to a device can be let go
/// of from anywhere — a cancelled task on the runtime, a track reader ending
/// — and waiting for a sound card to close from inside the reactor is the
/// kind of stall AGENTS says never to build. The thread notices the drop,
/// closes the stream, and ends on its own.
struct Worker<T = ()> {
    info: T,
    stop: Option<std::sync::mpsc::Sender<()>>,
}

impl<T: Send + 'static> Worker<T> {
    /// Run `build` on a fresh thread, start the stream it returns, and hold
    /// both until this worker is dropped. Returns once the stream is playing
    /// or the device refused, whichever comes first.
    fn start<F>(build: F) -> Result<Self, DeviceError>
    where
        F: FnOnce() -> Result<(cpal::Stream, T), DeviceError> + Send + 'static,
    {
        let (ready_tx, ready_rx) = std::sync::mpsc::channel::<Result<T, DeviceError>>();
        let (stop_tx, stop_rx) = std::sync::mpsc::channel::<()>();
        std::thread::Builder::new()
            .name("linger-audio".into())
            .spawn(move || {
                let (stream, info) = match build() {
                    Ok(built) => built,
                    Err(error) => {
                        let _ = ready_tx.send(Err(error));
                        return;
                    }
                };
                if let Err(error) = stream.play() {
                    let _ = ready_tx.send(Err(error.into()));
                    return;
                }
                let _ = ready_tx.send(Ok(info));
                // Sleeps until `stop` is used or dropped. The stream is
                // dropped here, on the thread that built it.
                let _ = stop_rx.recv();
                drop(stream);
            })?;
        let info = ready_rx.recv().map_err(|_| DeviceError::Gone)??;
        Ok(Self {
            info,
            stop: Some(stop_tx),
        })
    }
}

impl<T> Drop for Worker<T> {
    fn drop(&mut self) {
        // Dropping the sender is what wakes the thread.
        self.stop.take();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_stereo_frame_becomes_one_sample() {
        assert_eq!(
            downmix([100i16, 300, -50, -50].into_iter(), 2),
            vec![200, -50]
        );
    }

    #[test]
    fn a_partial_frame_waits_for_the_rest() {
        // Three samples of a stereo stream is one frame and a half; the half
        // is not a sample yet.
        assert_eq!(downmix([1i16, 3, 5].into_iter(), 2), vec![2]);
    }

    #[test]
    fn float_samples_map_onto_the_whole_range() {
        assert_eq!(from_f32(0.0), 0);
        assert_eq!(from_f32(1.0), i16::MAX);
        assert_eq!(from_f32(-1.0), -i16::MAX);
        assert_eq!(from_f32(2.0), i16::MAX, "out-of-range input must clamp");
        assert!((to_f32(i16::MIN) + 1.0).abs() < 1e-6);
    }

    #[test]
    fn resampling_a_constant_stays_constant_and_changes_length() {
        let mut linear = Linear::new(44_100, 48_000);
        let mut out = Vec::new();
        // Feed it in odd-sized pieces so the boundary carry is exercised.
        linear.push(&[1000i16; 441], &mut out);
        linear.push(&[1000i16; 300], &mut out);
        linear.push(&[1000i16; 141], &mut out);
        // 882 in at 44.1 kHz is 20 ms, which is 960 out at 48 kHz, give or
        // take the sample carried across the edge.
        assert!(
            (958..=961).contains(&out.len()),
            "got {} samples",
            out.len()
        );
        // The first two outputs lean on the zero carried in before any input
        // (they fall between it and the first real sample); all the rest is
        // the constant.
        assert!(out[2..].iter().all(|s| *s == 1000), "the level wandered");
    }

    #[test]
    fn resampling_at_the_same_rate_is_a_copy() {
        let mut linear = Linear::new(48_000, 48_000);
        let mut out = Vec::new();
        linear.push(&[10i16, 20, 30, 40], &mut out);
        // One sample of delay (the carried one), then the input verbatim.
        assert_eq!(out, vec![0, 10, 20, 30]);
        linear.push(&[50i16], &mut out);
        assert_eq!(out, vec![0, 10, 20, 30, 40]);
    }

    #[test]
    fn a_framer_cuts_frames_at_exactly_twenty_milliseconds() {
        let (tx, mut rx) = mpsc::channel(4);
        let mut framer = Framer::new(1, SAMPLE_RATE, tx);
        framer.push(std::iter::repeat_n(7i16, FRAME_SAMPLES - 1));
        assert!(rx.try_recv().is_err(), "a frame was cut short");
        framer.push(std::iter::once(7i16));
        let frame = rx.try_recv().expect("a whole frame");
        assert_eq!(frame.len(), FRAME_SAMPLES);
        assert!(frame.iter().all(|s| *s == 7));
    }

    /// Mixing is a sum with a ceiling: two loud peers must not wrap around
    /// into a click.
    #[test]
    fn mixing_sums_peers_and_clamps() {
        let lanes = Mutex::new(HashMap::new());
        for (peer, level) in [("a", 20_000i16), ("b", 20_000)] {
            lanes.lock().unwrap().insert(
                peer.to_string(),
                Lane {
                    queue: VecDeque::from(vec![level, -level, 100]),
                    resampler: None,
                },
            );
        }
        let mut out = [0i16; 8];
        mix(&lanes, 2, &mut out, |s| s);
        // Stereo: each mixed sample lands in both slots.
        assert_eq!(out[0], i16::MAX);
        assert_eq!(out[1], i16::MAX);
        assert_eq!(out[2], i16::MIN);
        assert_eq!(out[3], i16::MIN);
        assert_eq!(out[4], 200);
        assert_eq!(out[5], 200);
        // Both lanes ran dry: silence, not a stale sample.
        assert_eq!(out[6], 0);
        assert_eq!(out[7], 0);
    }

    // The two below need a real audio device and are run by hand
    // (`cargo test -- --ignored`). CI has no sound card, and a test that skips
    // itself quietly on a runner is not the same as one that passed.

    /// The microphone produces frames of the right shape, at roughly the
    /// right rate. What it heard is not checked — a quiet room is a valid
    /// microphone.
    #[tokio::test]
    #[ignore = "needs a real input device"]
    async fn the_microphone_produces_frames_in_real_time() {
        let microphone = Microphone::open().expect("open the default microphone");
        let started = std::time::Instant::now();
        for _ in 0..25 {
            let frame = tokio::time::timeout(std::time::Duration::from_secs(2), microphone.frame())
                .await
                .expect("a frame within two seconds")
                .expect("the microphone stopped");
            assert_eq!(frame.len(), FRAME_SAMPLES);
        }
        // 25 frames is half a second of audio; a device delivering them much
        // faster is making them up and much slower is dropping them.
        let took = started.elapsed().as_millis();
        assert!((350..=1500).contains(&took), "25 frames took {took} ms");
    }

    /// Half a second of a tone out of the speakers. Whether it was heard is a
    /// human check; what this proves is that the device opens, plays, and
    /// closes again without complaint.
    #[tokio::test]
    #[ignore = "needs a real output device, and makes a sound"]
    async fn the_speaker_plays_a_tone_and_closes() {
        let speaker = Speaker::open().expect("open the default speakers");
        let tone = crate::voice::audio::Tone::default();
        for _ in 0..25 {
            let frame = tone.frame().await.expect("a frame");
            speaker.play("test", &frame).await;
        }
        tokio::time::sleep(std::time::Duration::from_millis(300)).await;
        speaker.forget("test").await;
        drop(speaker);
    }
}
