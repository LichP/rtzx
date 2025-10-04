use hound;
use rodio::Source;
use std::io;
use std::sync::Arc;

use crate::tzx::{Machine, TzxData};
use crate::ui::commands::ConvertArgs;

pub fn run_convert(args: &ConvertArgs, machine: &Machine, tzx_data: &TzxData) -> io::Result<()> {
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: 48000,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut wav_writer = hound::WavWriter::create(&args.output_file_name, spec).expect("cannot open output wav file");

    let machine_arc = Arc::new(machine.clone());
    let mut start_pulse_high = true;
    for block in &tzx_data.blocks {
        let waveforms = block.get_waveforms(machine_arc.clone(), start_pulse_high);
        for waveform in waveforms {
            let source: Box<dyn Source + Send> = waveform;
            for sample in source {
                let val = (sample * i16::MAX as f32) as i16;
                wav_writer.write_sample(val).unwrap();
            }
        }
        start_pulse_high = block.next_block_start_pulse_high(start_pulse_high);
    }
    wav_writer.finalize().unwrap();
    return Ok(());
}
