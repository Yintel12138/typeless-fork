use anyhow::{Context, Result};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::{Arc, Mutex};
use std::time::Duration;

pub struct AudioRecorder {
    samples: Arc<Mutex<Vec<f32>>>,
    sample_rate: Arc<Mutex<u32>>,
    rms: Arc<Mutex<f32>>,
    stream: Option<cpal::Stream>,
}

// cpal::Stream is not Send, so we wrap the recorder to be explicitly Send + Sync
// The stream is only accessed from the owning thread.
unsafe impl Send for AudioRecorder {}
unsafe impl Sync for AudioRecorder {}

impl AudioRecorder {
    pub fn new() -> Self {
        Self {
            samples: Arc::new(Mutex::new(Vec::new())),
            sample_rate: Arc::new(Mutex::new(16000)),
            rms: Arc::new(Mutex::new(0.0)),
            stream: None,
        }
    }

    pub fn start_recording(&mut self) -> Result<()> {
        let host = cpal::default_host();
        let device = host
            .default_input_device()
            .context("No input audio device found")?;

        let config = device
            .default_input_config()
            .context("Failed to get default input config")?;

        let rate = config.sample_rate().0;
        *self.sample_rate.lock().unwrap() = rate;

        // Clear previous recording
        self.samples.lock().unwrap().clear();

        let samples_clone = Arc::clone(&self.samples);
        let rms_clone = Arc::clone(&self.rms);

        let err_fn = |err| log::error!("Audio stream error: {err}");

        let stream = match config.sample_format() {
            cpal::SampleFormat::F32 => device.build_input_stream(
                &config.into(),
                move |data: &[f32], _| {
                    let mut buf = samples_clone.lock().unwrap();
                    buf.extend_from_slice(data);
                    let rms = compute_rms(data);
                    *rms_clone.lock().unwrap() = rms;
                },
                err_fn,
                Some(Duration::from_secs(30)),
            )?,
            cpal::SampleFormat::I16 => device.build_input_stream(
                &config.into(),
                move |data: &[i16], _| {
                    let floats: Vec<f32> =
                        data.iter().map(|&s| s as f32 / i16::MAX as f32).collect();
                    let mut buf = samples_clone.lock().unwrap();
                    buf.extend_from_slice(&floats);
                    let rms = compute_rms(&floats);
                    *rms_clone.lock().unwrap() = rms;
                },
                err_fn,
                Some(Duration::from_secs(30)),
            )?,
            cpal::SampleFormat::U16 => device.build_input_stream(
                &config.into(),
                move |data: &[u16], _| {
                    let floats: Vec<f32> = data
                        .iter()
                        .map(|&s| (s as f32 / u16::MAX as f32) * 2.0 - 1.0)
                        .collect();
                    let mut buf = samples_clone.lock().unwrap();
                    buf.extend_from_slice(&floats);
                    let rms = compute_rms(&floats);
                    *rms_clone.lock().unwrap() = rms;
                },
                err_fn,
                Some(Duration::from_secs(30)),
            )?,
            fmt => anyhow::bail!("Unsupported sample format: {:?}", fmt),
        };

        stream.play().context("Failed to start audio stream")?;
        self.stream = Some(stream);
        Ok(())
    }

    pub fn stop_recording(&mut self) -> (Vec<f32>, u32) {
        // Drop the stream to stop capture
        self.stream = None;
        *self.rms.lock().unwrap() = 0.0;
        let samples = self.samples.lock().unwrap().clone();
        let rate = *self.sample_rate.lock().unwrap();
        (samples, rate)
    }

    pub fn get_rms(&self) -> f32 {
        *self.rms.lock().unwrap()
    }

    pub fn export_wav(samples: &[f32], sample_rate: u32) -> Result<Vec<u8>> {
        let spec = hound::WavSpec {
            channels: 1,
            sample_rate,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };

        let mut cursor = std::io::Cursor::new(Vec::new());
        {
            let mut writer =
                hound::WavWriter::new(&mut cursor, spec).context("Failed to create WAV writer")?;
            for &sample in samples {
                let s = (sample.clamp(-1.0, 1.0) * i16::MAX as f32) as i16;
                writer
                    .write_sample(s)
                    .context("Failed to write WAV sample")?;
            }
            writer.finalize().context("Failed to finalize WAV")?;
        }
        Ok(cursor.into_inner())
    }
}

fn compute_rms(samples: &[f32]) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }
    let sum_sq: f32 = samples.iter().map(|s| s * s).sum();
    (sum_sq / samples.len() as f32).sqrt()
}

impl Default for AudioRecorder {
    fn default() -> Self {
        Self::new()
    }
}
