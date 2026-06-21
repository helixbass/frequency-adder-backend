use std::f32::consts::PI;

use hound::{SampleFormat, WavSpec, WavWriter};
use tempdir::TempDir;
use uuid::Uuid;

use crate::wav_file;

const SAMPLE_RATE: u32 = 44100;

pub fn write_wav_file(tmp_dir: &TempDir) -> Uuid {
    // let uuid = Uuid::new_v4();
    let uuid = uuid::uuid!("A52691A1-64AA-40C5-AEA8-9FD8C67230C4");

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
        let sample_value = (sample_num * 432.0 * 2.0 * PI).sin();
        writer
            .write_sample((sample_value * amplitude) as i16)
            .unwrap();
    }
    writer.finalize().unwrap();

    uuid
}
