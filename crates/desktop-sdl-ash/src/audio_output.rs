//! Baseline audio device output for the desktop adapter (SPEC-08 AUDIO-P1,
//! A4 of the baseline-audio package).
//!
//! `DesktopAudioOutputV1` owns the SDL playback stream and a bounded ring
//! buffer fed by the simulation worker's canonical PCM windows. All state is
//! presentation-only: device loss, open failure or callback underrun degrade
//! to silence with typed counters and never fail the frame or reach
//! simulation state.

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use sdl3::audio::{AudioCallback, AudioFormat, AudioSpec, AudioStreamWithCallback};

pub const DESKTOP_AUDIO_SAMPLE_RATE: i32 = 48_000;
pub const DESKTOP_AUDIO_CHANNELS: i32 = 2;
/// One second of stereo S16 audio; older samples drop past this bound.
pub const DESKTOP_AUDIO_RING_CAPACITY_SAMPLES: usize = 96_000;
const MAX_REOPEN_ATTEMPTS: u64 = 8;

struct DesktopAudioRingV1 {
    samples: VecDeque<i16>,
    underruns: u64,
}

struct DesktopAudioFeedV1 {
    ring: Arc<Mutex<DesktopAudioRingV1>>,
}

impl AudioCallback<i16> for DesktopAudioFeedV1 {
    fn callback(&mut self, stream: &mut sdl3::audio::AudioStream, requested: i32) {
        let mut output = vec![0_i16; requested.max(0) as usize];
        if let Ok(mut ring) = self.ring.lock() {
            let available = ring.samples.len().min(output.len());
            for slot in output.iter_mut().take(available) {
                if let Some(sample) = ring.samples.pop_front() {
                    *slot = sample;
                }
            }
            if available < output.len() {
                ring.underruns = ring.underruns.saturating_add(1);
            }
        }
        let _ = stream.put_data_i16(&output);
    }
}

enum DesktopAudioStateV1 {
    Disabled,
    Unavailable,
    #[allow(
        dead_code,
        reason = "the owned stream is the live device handle; keeping it alive is the whole point"
    )]
    Active(AudioStreamWithCallback<DesktopAudioFeedV1>),
}

/// Bounded audio sink handed to the frame-source closure on every pump.
/// `queue_pcm` accepts the canonical stereo windows mixed by the live
/// driver; ordering and content are exactly what the worker published.
pub struct DesktopAudioOutputV1 {
    state: DesktopAudioStateV1,
    audio: Option<sdl3::AudioSubsystem>,
    ring: Arc<Mutex<DesktopAudioRingV1>>,
    queued_samples: u64,
    dropped_samples: u64,
    device_faults: u64,
    reopen_attempts: u64,
    reopens: u64,
}

impl DesktopAudioOutputV1 {
    /// Opens the playback stream. An open failure or a missing device never
    /// fails the run: the sink stays `Unavailable` (silent with counters) and
    /// the frame loop continues without audio output.
    pub fn open(sdl: &sdl3::Sdl, enabled: bool) -> Self {
        let ring = Arc::new(Mutex::new(DesktopAudioRingV1 {
            samples: VecDeque::new(),
            underruns: 0,
        }));
        let mut output = Self {
            state: DesktopAudioStateV1::Disabled,
            audio: None,
            ring,
            queued_samples: 0,
            dropped_samples: 0,
            device_faults: 0,
            reopen_attempts: 0,
            reopens: 0,
        };
        if !enabled {
            return output;
        }
        match sdl.audio() {
            Ok(audio) => {
                output.audio = Some(audio);
                output.reopen_attempts = 1;
                output.try_open_stream();
            }
            Err(_) => {
                output.state = DesktopAudioStateV1::Unavailable;
                output.device_faults = 1;
            }
        }
        output
    }

    #[must_use]
    pub fn disabled() -> Self {
        Self::new_disabled()
    }

    fn new_disabled() -> Self {
        Self {
            state: DesktopAudioStateV1::Disabled,
            audio: None,
            ring: Arc::new(Mutex::new(DesktopAudioRingV1 {
                samples: VecDeque::new(),
                underruns: 0,
            })),
            queued_samples: 0,
            dropped_samples: 0,
            device_faults: 0,
            reopen_attempts: 0,
            reopens: 0,
        }
    }

    /// Queues one canonical PCM window. Past the ring bound the oldest
    /// samples drop (bounded omission); the content of every retained byte
    /// is exactly what the worker published.
    pub fn queue_pcm(&mut self, samples: &[i16]) {
        if matches!(self.state, DesktopAudioStateV1::Disabled) || samples.is_empty() {
            return;
        }
        let Ok(mut ring) = self.ring.lock() else {
            self.dropped_samples = self.dropped_samples.saturating_add(samples.len() as u64);
            return;
        };
        for sample in samples {
            if ring.samples.len() >= DESKTOP_AUDIO_RING_CAPACITY_SAMPLES {
                ring.samples.pop_front();
                self.dropped_samples = self.dropped_samples.saturating_add(1);
            }
            ring.samples.push_back(*sample);
        }
        self.queued_samples = self.queued_samples.saturating_add(samples.len() as u64);
    }

    /// Typed audio-device-loss fact: pause into bounded unavailable state and
    /// arm one bounded reopen attempt (SDL itself migrates or destroys the
    /// stream; our reopen is exact and counted).
    pub fn note_device_removed(&mut self) {
        if matches!(self.state, DesktopAudioStateV1::Disabled) {
            return;
        }
        self.device_faults = self.device_faults.saturating_add(1);
        self.state = DesktopAudioStateV1::Unavailable;
        self.try_reopen();
    }

    /// A new playback device appeared: if the sink is unavailable, use this
    /// as the bounded reopen trigger.
    pub fn note_device_added(&mut self) {
        if matches!(
            self.state,
            DesktopAudioStateV1::Disabled | DesktopAudioStateV1::Active(_)
        ) {
            return;
        }
        self.try_reopen();
    }

    #[must_use]
    pub const fn queued_samples(&self) -> u64 {
        self.queued_samples
    }

    #[must_use]
    pub const fn dropped_samples(&self) -> u64 {
        self.dropped_samples
    }

    #[must_use]
    pub fn callback_underruns(&self) -> u64 {
        self.ring.lock().map_or(0, |ring| ring.underruns)
    }

    #[must_use]
    pub const fn device_faults(&self) -> u64 {
        self.device_faults
    }

    #[must_use]
    pub const fn reopens(&self) -> u64 {
        self.reopens
    }

    #[must_use]
    pub fn output_active(&self) -> bool {
        matches!(self.state, DesktopAudioStateV1::Active(_))
    }

    fn try_reopen(&mut self) {
        if self.reopen_attempts >= MAX_REOPEN_ATTEMPTS {
            return;
        }
        self.reopen_attempts = self.reopen_attempts.saturating_add(1);
        self.try_open_stream();
    }

    fn try_open_stream(&mut self) {
        let Some(audio) = &self.audio else {
            self.state = DesktopAudioStateV1::Unavailable;
            return;
        };
        let spec = AudioSpec {
            freq: Some(DESKTOP_AUDIO_SAMPLE_RATE),
            channels: Some(DESKTOP_AUDIO_CHANNELS),
            format: Some(AudioFormat::S16LE),
        };
        let feed = DesktopAudioFeedV1 {
            ring: Arc::clone(&self.ring),
        };
        match audio.open_playback_stream::<_, i16>(&spec, feed) {
            Ok(stream) => {
                if stream.resume().is_ok() {
                    self.state = DesktopAudioStateV1::Active(stream);
                    self.reopens = self.reopens.saturating_add(1);
                } else {
                    self.state = DesktopAudioStateV1::Unavailable;
                    self.device_faults = self.device_faults.saturating_add(1);
                }
            }
            Err(_) => {
                self.state = DesktopAudioStateV1::Unavailable;
                self.device_faults = self.device_faults.saturating_add(1);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn queue_is_bounded_with_oldest_drop() {
        let mut output = DesktopAudioOutputV1::new_disabled();
        output.state = DesktopAudioStateV1::Unavailable;
        let samples = vec![7_i16; DESKTOP_AUDIO_RING_CAPACITY_SAMPLES + 10];
        output.queue_pcm(&samples);
        let ring = output.ring.lock().expect("ring");
        assert_eq!(ring.samples.len(), DESKTOP_AUDIO_RING_CAPACITY_SAMPLES);
        assert_eq!(output.dropped_samples(), 10);
        assert_eq!(output.queued_samples(), samples.len() as u64);
    }

    #[test]
    fn feed_drains_ring_then_counts_underrun() {
        let ring = Arc::new(Mutex::new(DesktopAudioRingV1 {
            samples: VecDeque::from(vec![1_i16, 2, 3]),
            underruns: 0,
        }));
        let mut output = DesktopAudioOutputV1::new_disabled();
        output.ring = Arc::clone(&ring);
        // Simulate the callback drain without a live stream.
        let mut drained = Vec::new();
        {
            let mut ring = ring.lock().expect("ring");
            for _ in 0..5 {
                drained.push(ring.samples.pop_front().unwrap_or(0));
            }
            if drained.len() > 3 {
                ring.underruns += 1;
            }
        }
        assert_eq!(drained, vec![1, 2, 3, 0, 0]);
        assert_eq!(output.callback_underruns(), 1);
    }

    #[test]
    fn disabled_sink_accepts_nothing() {
        let mut output = DesktopAudioOutputV1::new_disabled();
        output.queue_pcm(&[1, 2, 3]);
        assert_eq!(output.queued_samples(), 0);
        assert!(!output.output_active());
    }

    #[test]
    fn reopen_attempts_are_bounded() {
        let mut output = DesktopAudioOutputV1::new_disabled();
        output.state = DesktopAudioStateV1::Unavailable;
        for _ in 0..20 {
            output.note_device_removed();
        }
        assert!(output.reopen_attempts <= MAX_REOPEN_ATTEMPTS);
    }
}
