use hound::{SampleFormat, WavSpec, WavWriter};
use obelisk_m0::rt_audit::{audio_allocation_count, reset_audio_allocation_count, AuditAllocator};
use obelisk_m0::telemetry::CallbackHistogram;
use obelisk_m0::{default_twelve_tet_tuning, M0Config, M0Engine};
use particelle_midi::{MidiEvent, MidiEventKind, NoteEvent, OfflineMidiReader, TimedMidiEvent};
use particelle_tuning::ScalaTuning;
use sha2::{Digest, Sha256};
use std::error::Error;
use std::path::PathBuf;
use std::time::{Duration, Instant};

#[global_allocator]
static GLOBAL_ALLOCATOR: AuditAllocator = AuditAllocator;

#[derive(Debug)]
struct Options {
    output: PathBuf,
    midi: Option<PathBuf>,
    scl: Option<PathBuf>,
    kbm: Option<PathBuf>,
    duration_seconds: f64,
    sample_rate: u32,
    block_size: usize,
    stress: bool,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            output: PathBuf::from("obelisk-m0.wav"),
            midi: None,
            scl: None,
            kbm: None,
            duration_seconds: 5.0,
            sample_rate: 48_000,
            block_size: 128,
            stress: false,
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let options = parse_options()?;
    let tuning = load_tuning(&options)?;
    let sample_rate = f64::from(options.sample_rate);
    let timeline = match &options.midi {
        Some(path) => OfflineMidiReader::from_bytes(&std::fs::read(path)?, sample_rate)?,
        None if options.stress => stress_timeline(sample_rate),
        None => demo_timeline(sample_rate),
    };

    let config = M0Config {
        sample_rate,
        ..M0Config::default()
    };
    let mut engine = M0Engine::new(config, tuning)?;
    let mut block = vec![[0.0_f32; 2]; options.block_size];
    let callback_budget = Duration::from_secs_f64(options.block_size as f64 / sample_rate);
    let total_frames = (options.duration_seconds * sample_rate).round() as u64;
    let mut timing = CallbackHistogram::new();
    let mut peak = 0.0_f32;
    let mut hash = Sha256::new();

    let spec = WavSpec {
        channels: 2,
        sample_rate: options.sample_rate,
        bits_per_sample: 32,
        sample_format: SampleFormat::Float,
    };
    let mut writer = WavWriter::create(&options.output, spec)?;
    let mut frame = 0_u64;

    reset_audio_allocation_count();
    while frame < total_frames {
        let frames_this_block =
            usize::try_from((total_frames - frame).min(options.block_size as u64))?;
        let events = timeline.block_events(frame, frames_this_block);
        let started = Instant::now();
        engine.process_block(&events, &mut block[..frames_this_block]);
        timing.observe(started.elapsed(), callback_budget);

        for stereo in &block[..frames_this_block] {
            for sample in stereo {
                peak = peak.max(sample.abs());
                hash.update(sample.to_le_bytes());
                writer.write_sample(*sample)?;
            }
        }
        frame += frames_this_block as u64;
    }
    writer.finalize()?;

    println!("rendered: {}", options.output.display());
    println!("frames: {total_frames} @ {} Hz", options.sample_rate);
    println!("sample sha256: {:x}", hash.finalize());
    println!("peak: {:.3} dBFS", 20.0 * peak.max(1.0e-12).log10());
    println!(
        "callback budget: p99 <= {:.1}% max {:.1}% across {} blocks",
        timing.percentile_ratio(0.99) * 100.0,
        timing.max_ratio() * 100.0,
        timing.samples()
    );
    println!("audio-thread allocations: {}", audio_allocation_count());
    Ok(())
}

fn parse_options() -> Result<Options, Box<dyn Error>> {
    let mut options = Options::default();
    let mut args = std::env::args().skip(1);
    while let Some(argument) = args.next() {
        match argument.as_str() {
            "-o" | "--output" => options.output = PathBuf::from(next_value(&mut args, &argument)?),
            "--midi" => options.midi = Some(PathBuf::from(next_value(&mut args, &argument)?)),
            "--scl" => options.scl = Some(PathBuf::from(next_value(&mut args, &argument)?)),
            "--kbm" => options.kbm = Some(PathBuf::from(next_value(&mut args, &argument)?)),
            "--duration" => options.duration_seconds = next_value(&mut args, &argument)?.parse()?,
            "--sample-rate" => options.sample_rate = next_value(&mut args, &argument)?.parse()?,
            "--block-size" => options.block_size = next_value(&mut args, &argument)?.parse()?,
            "--stress" => options.stress = true,
            "-h" | "--help" => {
                print_help();
                std::process::exit(0);
            }
            unknown => return Err(format!("unknown option '{unknown}'").into()),
        }
    }
    if !options.duration_seconds.is_finite() || options.duration_seconds <= 0.0 {
        return Err("duration must be finite and greater than zero".into());
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
        "obelisk-m0-render\n\
         \n\
         Usage: obelisk-m0-render [options]\n\
         \n\
           -o, --output PATH       Output float WAV (default: obelisk-m0.wav)\n\
               --midi PATH         Optional Standard MIDI File\n\
               --scl PATH          Optional Scala scale\n\
               --kbm PATH          Optional Scala keyboard mapping\n\
               --duration SECONDS  Render duration (default: 5)\n\
               --sample-rate HZ    Sample rate (default: 48000)\n\
               --block-size FRAMES Callback size (default: 128)\n\
               --stress            Render a sustained 16-voice CPU exercise"
    );
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

fn demo_timeline(sample_rate: f64) -> OfflineMidiReader {
    let frame = |seconds: f64| (seconds * sample_rate).round() as u64;
    let mut events = Vec::new();

    for (channel, note, timbre) in [(2_u8, 48_u8, 0.22), (3, 55, 0.52), (4, 64, 0.78)] {
        events.push(timed(
            0,
            MidiEventKind::ControlChange {
                channel,
                cc: 74,
                value: timbre,
            },
        ));
        events.push(timed(
            0,
            MidiEventKind::ChannelPressure {
                channel,
                value: 0.18,
            },
        ));
        events.push(timed(
            frame(0.01),
            MidiEventKind::Note(NoteEvent {
                channel,
                note,
                velocity: 0.78,
                is_on: true,
            }),
        ));
        events.push(timed(
            frame(1.0),
            MidiEventKind::ChannelPressure {
                channel,
                value: 0.82,
            },
        ));
        events.push(timed(
            frame(2.0),
            MidiEventKind::ControlChange {
                channel,
                cc: 74,
                value: (timbre + 0.2_f64).min(1.0_f64),
            },
        ));
        events.push(timed(
            frame(3.2),
            MidiEventKind::Note(NoteEvent {
                channel,
                note,
                velocity: 0.35,
                is_on: false,
            }),
        ));
    }
    OfflineMidiReader::new(events)
}

fn stress_timeline(sample_rate: f64) -> OfflineMidiReader {
    let frame = |seconds: f64| (seconds * sample_rate).round() as u64;
    let mut events = Vec::new();
    for channel in 2_u8..=16 {
        let note = 35 + channel;
        events.push(timed(
            0,
            MidiEventKind::ControlChange {
                channel,
                cc: 74,
                value: 0.45 + f64::from(channel - 2) / 40.0,
            },
        ));
        events.push(timed(
            0,
            MidiEventKind::ChannelPressure {
                channel,
                value: 0.65,
            },
        ));
        events.push(timed(
            frame(0.01),
            MidiEventKind::Note(NoteEvent {
                channel,
                note,
                velocity: 0.65,
                is_on: true,
            }),
        ));
    }

    // Reassign one MPE channel while its previous note is releasing. This
    // exercises all sixteen engine slots while validating detached release state.
    events.push(timed(
        frame(0.5),
        MidiEventKind::Note(NoteEvent {
            channel: 2,
            note: 37,
            velocity: 0.1,
            is_on: false,
        }),
    ));
    events.push(timed(
        frame(0.51),
        MidiEventKind::Note(NoteEvent {
            channel: 2,
            note: 72,
            velocity: 0.65,
            is_on: true,
        }),
    ));
    OfflineMidiReader::new(events)
}

fn timed(frame: u64, kind: MidiEventKind) -> TimedMidiEvent {
    TimedMidiEvent {
        frame,
        event: MidiEvent {
            frame_offset: 0,
            kind,
        },
    }
}
