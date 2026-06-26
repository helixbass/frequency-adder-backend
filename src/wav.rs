use std::f32::consts::PI;

use hound::{SampleFormat, WavSpec, WavWriter};
use tempdir::TempDir;
use uuid::Uuid;

use crate::{types::Frequencies, wav_file};

const SAMPLE_RATE: u32 = 44100;
const SAMPLE_RATE_AS_FLOAT: f32 = 44100.0;

pub fn write_wav_file(frequencies: &Frequencies, tmp_dir: &TempDir, uuid: Uuid) {
    let multiplier = {
        let max_abs_value = get_max_abs_value(frequencies);
        1.0 / max_abs_value
    };

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

    let amplitude = i16::MAX as f32;
    for_each_sample_num(frequencies, |sample_num: u32, frequencies: &Frequencies| {
        let added_frequencies_value =
            get_added_frequencies_value(get_sample_fraction(sample_num), frequencies);
        let normalized = added_frequencies_value * multiplier;
        writer
            .write_sample((normalized * amplitude) as i16)
            .unwrap();
    });
    writer.finalize().unwrap();
}

fn get_max_abs_value(frequencies: &Frequencies) -> f32 {
    let mut max_abs_value_seen: f32 = 0.0;
    for_each_sample_num(frequencies, |sample_num: u32, frequencies: &Frequencies| {
        let this_added_value =
            get_added_frequencies_value(get_sample_fraction(sample_num), frequencies);
        let this_added_value_abs = this_added_value.abs();
        if this_added_value_abs > max_abs_value_seen {
            max_abs_value_seen = this_added_value_abs;
        }
    });
    max_abs_value_seen
}

fn get_added_frequencies_value(sample_fraction: f32, frequencies: &Frequencies) -> f32 {
    frequencies
        .into_iter()
        .map(|frequency| {
            get_frequency_sin_amplitude(sample_fraction, frequency.frequency as f32)
                * (frequency.magnitude as f32)
        })
        .sum()
}

fn for_each_sample_num(frequencies: &Frequencies, mut callback: impl FnMut(u32, &Frequencies)) {
    for sample_num in 0..SAMPLE_RATE {
        callback(sample_num, frequencies);
    }
}

fn get_sample_fraction(sample_num: u32) -> f32 {
    sample_num as f32 / SAMPLE_RATE_AS_FLOAT
}

fn get_frequency_sin_amplitude(frequency: f32, sample_fraction: f32) -> f32 {
    (sample_fraction * frequency * 2.0 * PI).sin()
}
