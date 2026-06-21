use std::f32::consts::PI;

use hound::{SampleFormat, WavSpec, WavWriter};
use tempdir::TempDir;
use uuid::Uuid;

use crate::wav_file;

const SAMPLE_RATE: u32 = 44100;

pub fn write_wav_file(frequency: f32, tmp_dir: &TempDir, uuid: Uuid) {
    // TODO: do I want hound to be async?
    let mut writer = WavWriter::create(
        wav_file::fs_path(tmp_dir, uuid),
        WavSpec {
            channels: 1,
            sample_rate: SAMPLE_RATE,
            bits_per_sample: 16,
            sample_format: SampleFormat::Int,
        },
    )
    .unwrap();

    let sample_rate_as_float = SAMPLE_RATE as f32;
    let amplitude = i16::MAX as f32;
    for sample_num in (0..SAMPLE_RATE).map(|sample_num| sample_num as f32 / sample_rate_as_float) {
        let sample_value = (sample_num * frequency * 2.0 * PI).sin();
        writer
            .write_sample((sample_value * amplitude) as i16)
            .unwrap();
    }
    writer.finalize().unwrap();
}
