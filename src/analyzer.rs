//! Microphone processing and analyzing
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, Host, Stream, StreamConfig};
use pitch_detector::{
    note::{detect_note, NoteDetectionResult},
    pitch::{HannedFftDetector, PitchDetector},
};
use ringbuf::{Consumer, HeapRb, SharedRb};
use std::mem::MaybeUninit;
use std::sync::Arc;

/// Processes and analyzes microphone input.
#[allow(dead_code)]
pub struct MicrophoneAnalyzer {
    sample_buffer: Vec<f64>,
    host: Host,
    mic: Device,
    mic_config: StreamConfig,
    mic_stream: Stream,
    mic_consumer: Consumer<f64, Arc<SharedRb<f64, Vec<MaybeUninit<f64>>>>>,
    /// Cached for increased performance in `update`.
    back_buffer: Vec<f64>,
}

impl MicrophoneAnalyzer {
    pub fn new() -> Self {
        // HACK: Correctly return errors

        let host = cpal::default_host();
        let mic = host
            .default_input_device()
            .expect("Unable to get default input device.");
        let mic_config: StreamConfig = mic
            .default_input_config()
            .expect("Unable to get default input config.")
            .into();

        // HACK: Don't hardcode record range
        let num_samples = (mic_config.sample_rate.0 as f64 * 0.1) as usize; // Records the last 0.1th of a second

        let ring = HeapRb::<f64>::new(num_samples);
        let (mut producer, consumer) = ring.split();

        fn err_fn(err: cpal::StreamError) {
            eprintln!("Error occurred on stream: {}", err);
        }

        let mic_stream = mic
            .build_input_stream(
                &mic_config,
                move |data: &[f32], _: &cpal::InputCallbackInfo| {
                    // `allow`ed because there should not be any issues if the data cannot be pushed.
                    #[allow(unused_must_use)]
                    for sample in data {
                        producer.push(*sample as f64);
                    }
                },
                err_fn,
            )
            .expect("Unable to build input stream.");

        mic_stream.play().expect("Unable to start input stream.");

        Self {
            sample_buffer: vec![0.0; num_samples],
            host,
            mic,
            mic_config,
            mic_stream,
            mic_consumer: consumer,
            back_buffer: vec![0.0; num_samples],
        }
    }
}

impl MicrophoneAnalyzer {
    fn update(&mut self) {
        // TODO: Look for more ways to optimize this function
        let size = self.mic_consumer.pop_slice(&mut self.back_buffer);
        let sample_buffer_len = self.sample_buffer.len();
        self.sample_buffer.copy_within(size..sample_buffer_len, 0);
        self.sample_buffer[(sample_buffer_len - size)..sample_buffer_len]
            .copy_from_slice(&self.back_buffer[0..size]);

        // let mut back_buffer = vec![0.0; self.mic_consumer.len()];
        // self.mic_consumer.pop_slice(&mut back_buffer);
        // self.sample_buffer.drain(0..back_buffer.len());
        // self.sample_buffer
        //     .append(&mut back_buffer.into_iter().map(|x| x as f64).collect());
    }

    fn power(&self) -> f64 {
        fn root_mean_square(vec: &[f64]) -> f64 {
            let sum_squares = vec.iter().fold(0.0, |acc, &x| acc + x.powf(2.0));
            return ((sum_squares) / (vec.len() as f64)).sqrt();
        }

        // HACK: Don't hardcode microphone power range
        root_mean_square(&self.sample_buffer[0..(self.sample_buffer.len() / 8)])
    }

    fn pitch(&self) -> f64 {
        let mut detector = HannedFftDetector::default();
        detector
            .detect_pitch(&self.sample_buffer, self.mic_config.sample_rate.0 as f64)
            .unwrap_or(0.0)
    }

    fn note(&self) -> Option<NoteDetectionResult> {
        let mut detector = HannedFftDetector::default();
        detect_note(
            &self.sample_buffer,
            &mut detector,
            self.mic_config.sample_rate.0 as f64,
        )
    }

    pub fn update_and_analyze(&mut self) -> AnalyzerResults {
        self.update();

        let note = self.note();
        let pitch = self.pitch();
        let power = self.power();

        AnalyzerResults { note, pitch, power }
    }
}

#[derive(Clone)]
pub struct AnalyzerResults {
    pub note: Option<NoteDetectionResult>,
    pub pitch: f64,
    pub power: f64,
}
