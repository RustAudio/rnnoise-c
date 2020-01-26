// Download, build, and install https://github.com/xiph/rnnoise

extern crate audrey;
extern crate rnnoise_c;

use std::env::args;
use std::fs::File;

use audrey::read::Reader;
use audrey::sample::interpolate::{Converter, Linear};
use audrey::sample::signal::{from_iter, Signal};
use rnnoise_c::DenoiseState;

// RNNoise assumes audio is 16-bit mono with a 48 KHz sample rate
const SAMPLE_RATE :u32 = 48_000;

fn main() {
    let audio_file_path = args().nth(1)
        .expect("Please specify an audio file to run RNNoise on");
    let audio_file = File::open(audio_file_path).unwrap();
    let mut reader = Reader::new(audio_file).unwrap();
    let desc = reader.description();
    assert_eq!(1, desc.channel_count(),
        "The channel count is required to be one, at least for now");

    // Obtain the buffer of samples
    let audio_buf :Vec<_> = if desc.sample_rate() == SAMPLE_RATE {
        reader.samples().map(|s| s.unwrap()).collect()
    } else {
        // We need to interpolate to the target sample rate
        let interpolator = Linear::new([0i16], [0]);
        let conv = Converter::from_hz_to_hz(
            from_iter(reader.samples::<i16>().map(|s| [s.unwrap()])),
            interpolator,
            desc.sample_rate() as f64,
            SAMPLE_RATE as f64);
        conv.until_exhausted().map(|v| v[0]).collect()
    };

    let rnnoise = DenoiseState::new();
}
