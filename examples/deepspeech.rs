// In order to compile and use this example you must first follow the
// preparation directions on https://github.com/RustAudio/deepspeech-rs

extern crate audrey;
extern crate deepspeech;
extern crate hound;
extern crate rnnoise_c;
extern crate structopt;

use std::fs::File;
use std::path::Path;

use audrey::read::Reader;
use audrey::sample::interpolate::{Converter, Linear};
use audrey::sample::signal::{from_iter, Signal};
use deepspeech::Model;
use rnnoise_c::{DenoiseState, FRAME_SIZE};
use structopt::StructOpt;

// RNNoise assumes audio is 16-bit mono with a 48 KHz sample rate
const RNNOISE_SAMPLE_RATE :u32 = 48_000;

// The DeepSpeech model has been trained on 16 KHz
const DEEPSPEECH_SAMPLE_RATE :u32 = 16_000;

// Deepspeech constants
const BEAM_WIDTH :u16 = 500;
const LM_WEIGHT :f32 = 0.75;
const VALID_WORD_COUNT_WEIGHT :f32 = 1.85;

#[derive(StructOpt, Debug, Clone)]
#[structopt(name = "client")]
pub struct Configuration {
    /// Verbose mode
    #[structopt(short, long)]
    verbose: bool,
    /// Input audio file
    #[structopt(short, long, required=true)]
    input: String,
    /// Output audio file
    #[structopt(short, long, required=true)]
    output: String,
    /// Deepspeech model directory
    #[structopt(short, long, required=true)]
    model: String,
}

fn main() {
    let configuration = Configuration::from_args();
    if configuration.verbose {
        println!("Configuration: {:?}", configuration);
    }

    // Load input audio file
    let audio_file = File::open(&configuration.input).unwrap();
    let mut reader = Reader::new(audio_file).unwrap();
    let desc = reader.description();
    if configuration.verbose {
        println!("description: {:?}", &desc);
    }
    assert_eq!(1, desc.channel_count(),
        "The channel count is required to be one, at least for now");

    // Obtain the buffer of samples
    let mut audio_buf :Vec<_> = if desc.sample_rate() == RNNOISE_SAMPLE_RATE {
        reader.samples::<f32>().map(|s| s.unwrap()).collect()
    } else {
        // We need to interpolate to the target sample rate
        let interpolator = Linear::new([0f32], [0.0]);
        let conv = Converter::from_hz_to_hz(
            from_iter(reader.samples::<f32>().map(|s| [s.unwrap()])),
            interpolator,
            desc.sample_rate() as f64,
            RNNOISE_SAMPLE_RATE as f64);
        conv.until_exhausted().map(|v| v[0]).collect()
    };

    if configuration.verbose {
        println!("{} length: {}", &configuration.input, audio_buf.len());
    }

    // The library requires each frame be exactly FRAME_SIZE, so we append
    // some zeros to be sure the final frame is sufficiently long.
    let padding = audio_buf.len() % FRAME_SIZE;
    if padding > 0 {
        let mut pad: Vec<f32> = vec![0.0; FRAME_SIZE - padding];
        audio_buf.append(&mut pad);
        if configuration.verbose {
            println!("padded audio file with {} characters", padding);
        }
    }
    let mut denoised_buffer: Vec<f32> = vec![];
    let mut rnnoise = DenoiseState::new();
    let mut denoised_chunk: Vec<f32> = vec![0.0; FRAME_SIZE];
    let buffers = audio_buf[..].chunks(FRAME_SIZE);
    for buffer in buffers {
        rnnoise.process_frame_mut(&buffer, &mut denoised_chunk[..]);
        denoised_buffer.extend_from_slice(&mut denoised_chunk);
    }

    if configuration.verbose {
        println!("{} length: {}", &configuration.output, denoised_buffer.len());
    }
    assert_eq!(audio_buf.len(), denoised_buffer.len());

    // Write denoised buffer into output file
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: RNNOISE_SAMPLE_RATE,
        bits_per_sample: 32,
        sample_format: hound::SampleFormat::Float,
    };
    let opt_wav_writer = hound::WavWriter::create(&configuration.output, spec);
    let mut wav_writer = opt_wav_writer.expect("failed to create wav file");
    denoised_buffer.iter().for_each(|i| wav_writer.write_sample(*i).expect("failed to write to wav file"));
    wav_writer.finalize().unwrap();

    // Set up DeepSpeech model
    let dir_path = Path::new(&configuration.model);
    let mut m = Model::load_from_files(
        &dir_path.join("output_graph.pb"),
        BEAM_WIDTH).unwrap();
    m.enable_decoder_with_lm(
        &dir_path.join("lm.binary"),
        &dir_path.join("trie"),
        LM_WEIGHT,
        VALID_WORD_COUNT_WEIGHT);

    // Use denoised audio file with Deepspeech
    let denoised_audio_file = File::open(&configuration.output).unwrap();
    let mut denoised_reader = Reader::new(denoised_audio_file).unwrap();
    let denoised_desc = denoised_reader.description();
    if configuration.verbose {
        println!("denoised description: {:?}", &denoised_desc);
    }
    assert_eq!(1, denoised_desc.channel_count(),
        "The channel count is required to be one, at least for now");

    // Obtain the buffer of samples
    let denoised_audio_buf :Vec<_> = {
        // We need to interpolate to the target sample rate
        let interpolator = Linear::new([0i16], [0]);
        let conv = Converter::from_hz_to_hz(
            from_iter(denoised_reader.samples::<i16>().map(|s| [s.unwrap()])),
            interpolator,
            denoised_desc.sample_rate() as f64,
            DEEPSPEECH_SAMPLE_RATE as f64);
        conv.until_exhausted().map(|v| v[0]).collect()
    };

    // Run the speech to text algorithm
    let result = m.speech_to_text(&denoised_audio_buf).unwrap();

    // Output the result
    println!("{}", result);
}
