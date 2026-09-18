use obelisk_m0::rt_audit::AuditAllocator;
use obelisk_m0::{default_twelve_tet_tuning, M0Config, M0Engine};
use particelle_io::hardware::{HardwareConfig, HardwareHost};
use particelle_midi::realtime::RealtimeMidiHost;
use particelle_midi::MidiEvent;
use particelle_tuning::ScalaTuning;
use std::error::Error;
use std::path::PathBuf;

const MIDI_QUEUE_CAPACITY: usize = 8192;

#[global_allocator]
static GLOBAL_ALLOCATOR: AuditAllocator = AuditAllocator;

#[derive(Debug)]
struct Options {
    midi_port: Option<String>,
    audio_device: Option<String>,
    scl: Option<PathBuf>,
    kbm: Option<PathBuf>,
    sample_rate: u32,
    block_size: usize,
    list_devices: bool,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            midi_port: None,
            audio_device: None,
            scl: None,
            kbm: None,
            sample_rate: 48_000,
            block_size: 128,
            list_devices: false,
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let options = parse_options()?;
    if options.list_devices {
        list_devices()?;
        return Ok(());
    }

    let tuning = load_tuning(&options)?;
    let config = M0Config {
        sample_rate: f64::from(options.sample_rate),
        ..M0Config::default()
    };
    let mut engine = M0Engine::new(config, tuning)?;
    let mut midi_host = RealtimeMidiHost::new();
    let (midi_name, mut midi_queue) =
        midi_host.start_ring(options.midi_port.as_deref(), MIDI_QUEUE_CAPACITY)?;
    eprintln!("→ MPE input: {midi_name}");

    let hardware = HardwareHost::new(HardwareConfig {
        device_name: options.audio_device,
        n_channels: 2,
        sample_rate: f64::from(options.sample_rate),
        block_size: options.block_size,
        ..HardwareConfig::default()
    });

    let mut scratch = vec![[0.0_f32; 2]; options.block_size];
    let mut events = Vec::<MidiEvent>::with_capacity(MIDI_QUEUE_CAPACITY);
    let mut last_dropped_count = 0_u64;
    hardware.run(move |interleaved| {
        events.clear();
        while let Some(mut event) = midi_queue.try_pop() {
            event.frame_offset = 0;
            if events.len() < events.capacity() {
                events.push(event);
            }
        }
        if midi_queue.take_emergency_all_notes_off() {
            engine.all_notes_off();
        }

        let dropped = midi_queue.dropped_events();
        if dropped != last_dropped_count {
            // Do not log on the audio thread. Remember the count so a future
            // control-thread status view can publish it.
            last_dropped_count = dropped;
        }

        let mut first_chunk = true;
        for output_chunk in interleaved.chunks_mut(options.block_size * 2) {
            let frame_count = output_chunk.len() / 2;
            if frame_count == 0 {
                continue;
            }
            let block_events = if first_chunk { events.as_slice() } else { &[] };
            engine.process_block(block_events, &mut scratch[..frame_count]);
            for (target, stereo) in output_chunk
                .chunks_exact_mut(2)
                .zip(&scratch[..frame_count])
            {
                target[0] = stereo[0];
                target[1] = stereo[1];
            }
            first_chunk = false;
        }
    })?;
    Ok(())
}

fn parse_options() -> Result<Options, Box<dyn Error>> {
    let mut options = Options::default();
    let mut args = std::env::args().skip(1);
    while let Some(argument) = args.next() {
        match argument.as_str() {
            "--midi" => options.midi_port = Some(next_value(&mut args, &argument)?),
            "--audio" => options.audio_device = Some(next_value(&mut args, &argument)?),
            "--scl" => options.scl = Some(PathBuf::from(next_value(&mut args, &argument)?)),
            "--kbm" => options.kbm = Some(PathBuf::from(next_value(&mut args, &argument)?)),
            "--sample-rate" => options.sample_rate = next_value(&mut args, &argument)?.parse()?,
            "--block-size" => options.block_size = next_value(&mut args, &argument)?.parse()?,
            "--list" => options.list_devices = true,
            "-h" | "--help" => {
                print_help();
                std::process::exit(0);
            }
            unknown => return Err(format!("unknown option '{unknown}'").into()),
        }
    }
    if options.sample_rate == 0 || options.block_size == 0 {
        return Err("sample rate and block size must be greater than zero".into());
    }
    if options.kbm.is_some() && options.scl.is_none() {
        return Err("--kbm requires --scl".into());
    }
    Ok(options)
}

fn next_value(
    args: &mut impl Iterator<Item = String>,
    option: &str,
) -> Result<String, Box<dyn Error>> {
    args.next()
        .ok_or_else(|| format!("missing value for {option}").into())
}

fn print_help() {
    println!(
        "obelisk-m0-live\n\
         \n\
         Usage: obelisk-m0-live [options]\n\
         \n\
               --list              List audio and MIDI devices\n\
               --midi NAME         MPE MIDI input; first input by default\n\
               --audio NAME        Audio output; system default by default\n\
               --scl PATH          Optional Scala scale\n\
               --kbm PATH          Optional Scala keyboard mapping\n\
               --sample-rate HZ    Sample rate (default: 48000)\n\
               --block-size FRAMES Callback size (default: 128)"
    );
}

fn list_devices() -> Result<(), Box<dyn Error>> {
    println!("Audio outputs:");
    for name in HardwareHost::list_devices()? {
        println!("  {name}");
    }
    println!("MIDI inputs:");
    for name in RealtimeMidiHost::list_input_ports()? {
        println!("  {name}");
    }
    Ok(())
}

fn load_tuning(options: &Options) -> Result<ScalaTuning, Box<dyn Error>> {
    let Some(scl_path) = &options.scl else {
        return Ok(default_twelve_tet_tuning());
    };
    let scl = std::fs::read_to_string(scl_path)?;
    let kbm = options
        .kbm
        .as_ref()
        .map(std::fs::read_to_string)
        .transpose()?;
    ScalaTuning::from_text(&scl, kbm.as_deref(), 440.0).map_err(Into::into)
}
