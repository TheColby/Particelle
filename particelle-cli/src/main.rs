use anyhow::{Context, Result};
use clap::{Parser, Subcommand};

/// Particelle — granular synthesis engine command-line interface.
///
/// All engine behavior is configured via YAML. This CLI is a thin wrapper
/// over the engine API. No audio logic lives here.
#[derive(Parser)]
#[command(
    name = "particelle",
    version,
    about = "Particelle — a research-grade granular synthesis engine for immersive and microtonal composition.",
    long_about = "\
Particelle is a 64-bit, surround-native, microtonal-first granular synthesis engine \
written entirely in Rust. All behavior is declared in YAML configuration files. \
Every parameter is a signal. Every render is deterministic.\n\n\
Supported tuning systems: 12-TET, arbitrary EDO, Just Intonation (rational ratios), Scala (.scl/.kbm).\n\
Supported layouts: stereo, 5.1, 7.1.4, arbitrary discrete multichannel (spherical or Cartesian).\n\
Window library: 35+ types (Hann, Kaiser, DPSS, Planck taper, KBD, and more).\n\
Parameter system: composable signal graphs with curves, MIDI, MPE, and arithmetic.",
    after_help = "\
QUICK START:\n\
    particelle init > patch.yaml          Generate a starter patch\n\
    particelle validate patch.yaml        Check for schema errors\n\
    particelle render patch.yaml -o out.wav --duration 10.0\n\
                                          Render 10 seconds to WAV\n\
    particelle run patch.yaml             Stream to hardware in realtime\n\n\
EXAMPLES:\n\
    # Render a 31-EDO drone for 60 seconds at 96kHz\n\
    particelle render drone.yaml -o drone.wav --duration 60.0\n\n\
    # Timestretch a file 4× (position driven by a linear curve)\n\
    particelle render stretch.yaml -o stretched.wav --duration 16.0\n\n\
    # Render and verify determinism with SHA-256 hash\n\
    particelle render patch.yaml -o out.wav --duration 5.0 --hash\n\n\
    # Preview a JSON curve at 1000-point resolution\n\
    particelle curve curves/density.json --resolution 1000\n\n\
    # Override sample rate for a batch experiment\n\
    particelle set patch.yaml engine.sample_rate 96000 > patch_96k.yaml\n\n\
DOCUMENTATION:\n\
    https://github.com/TheColby/Particelle",
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Validate a YAML patch file and report all schema errors.
    ///
    /// Checks engine config, layout, tuning, cloud parameters, window types,
    /// and routing bindings. Exits with code 1 if any errors are found.
    #[command(
        after_help = "EXAMPLES:\n    particelle validate patch.yaml\n    particelle validate experiments/microtonal_drone.yaml"
    )]
    Validate {
        /// Path to the YAML configuration file to validate.
        patch: String,
    },

    /// Render a patch to an audio file (offline, deterministic).
    ///
    /// Processes the full grain engine offline and writes a multichannel WAV file.
    /// Output is byte-identical across runs with equal inputs. Useful for batch
    /// rendering, regression testing, and non-realtime composition.
    #[command(
        after_help = "\
EXAMPLES:\n\
    particelle render shimmer.yaml -o shimmer.wav --duration 8.0\n\
    particelle render patch.yaml -o out.wav -d 60.0 --hash\n\n\
NOTES:\n\
    The output channel count matches the layout declared in the YAML patch.\n\
    Sample rate and block size are taken from engine config.\n\
    Use --hash to print a SHA-256 digest for deterministic regression tests."
    )]
    Render {
        /// Path to the YAML configuration file.
        patch: String,

        /// Output audio file path (WAV format).
        #[arg(short, long, help = "Output WAV file path")]
        output: String,

        /// Render duration in seconds.
        #[arg(short, long, default_value = "10.0", help = "Duration in seconds to render")]
        duration: f64,

        /// Print a deterministic SHA-256 hash of the output audio data.
        /// Useful for verifying byte-identical renders across runs.
        #[arg(long, help = "Print SHA-256 hash of output for regression testing")]
        hash: bool,
    },

    /// Run a patch in realtime using the configured hardware audio device.
    ///
    /// Opens a multichannel output stream via CPAL, matching the channel count
    /// and sample rate declared in the patch. The audio callback runs on a
    /// dedicated thread with zero heap allocation. Press Ctrl+C to stop.
    #[command(
        after_help = "\
EXAMPLES:\n\
    particelle run shimmer.yaml\n\
    particelle run immersive_7.1.4.yaml\n\n\
NOTES:\n\
    Hardware device is selected by name in the patch hardware section.\n\
    If no device is specified, the system default output is used.\n\
    MIDI input is ingested off the audio thread via lock-free queue."
    )]
    Run {
        /// Path to the YAML configuration file.
        patch: String,
    },

    /// Generate a default YAML patch to stdout.
    ///
    /// Produces a complete, valid starter patch with sensible defaults.
    /// Redirect to a file and edit to begin composing.
    #[command(
        after_help = "\
EXAMPLES:\n\
    particelle init > my_patch.yaml\n\
    particelle init -n 12 > atmos_patch.yaml    # 12-channel layout"
    )]
    Init {
        /// Number of output channels to include in the layout.
        #[arg(short = 'n', long, default_value = "2", help = "Number of output channels (e.g., 2 for stereo, 12 for Atmos)")]
        channels: usize,
    },

    /// Inspect, evaluate, and print a JSON curve file.
    ///
    /// Compiles the curve and prints (x, y) sample pairs to stdout in TSV format.
    /// Pipe to a plotting tool or spreadsheet for visualization.
    #[command(
        after_help = "\
EXAMPLES:\n\
    particelle curve curves/density.json\n\
    particelle curve curves/position.json --resolution 1000\n\
    particelle curve curves/envelope.json -r 500 > envelope.tsv"
    )]
    Curve {
        /// Path to the JSON curve file.
        curve: String,

        /// Number of evaluation points to print across the curve domain.
        #[arg(short, long, default_value = "64", help = "Number of sample points to evaluate")]
        resolution: usize,
    },

    /// Override a single parameter value in a patch.
    ///
    /// Reads the YAML patch, substitutes the value at the given dot-separated
    /// parameter path, and prints the modified YAML to stdout. Useful for
    /// scripted batch experiments and parameter sweeps.
    #[command(
        after_help = "\
EXAMPLES:\n\
    particelle set patch.yaml engine.sample_rate 96000 > patch_96k.yaml\n\
    particelle set patch.yaml tuning.steps 19 > patch_19edo.yaml\n\n\
PARAMETER PATHS:\n\
    engine.sample_rate    Engine sample rate (Hz)\n\
    engine.block_size     Block size (samples)\n\
    tuning.steps          EDO step count\n\
    clouds[0].density     First cloud grain density"
    )]
    Set {
        /// Path to the YAML configuration file.
        patch: String,

        /// Canonical dot-separated parameter path (e.g., `engine.sample_rate`).
        param: String,

        /// New value to assign.
        value: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Validate { patch } => {
            cmd_validate(&patch)?;
        }
        Commands::Render { patch, output, duration, hash } => {
            cmd_render(&patch, &output, duration, hash)?;
        }
        Commands::Run { patch } => {
            cmd_run(&patch)?;
        }
        Commands::Init { channels } => {
            cmd_init(channels)?;
        }
        Commands::Curve { curve, resolution } => {
            cmd_curve(&curve, resolution)?;
        }
        Commands::Set { patch, param, value } => {
            cmd_set(&patch, &param, &value)?;
        }
    }

    Ok(())
}

fn cmd_validate(patch_path: &str) -> Result<()> {
    let yaml = std::fs::read_to_string(patch_path)
        .with_context(|| format!("Cannot read '{}'", patch_path))?;
    let config: particelle_schema::ParticelleConfig = serde_yaml::from_str(&yaml)
        .with_context(|| "YAML parse error")?;
    let errors = particelle_schema::validate(&config);
    if errors.is_empty() {
        let n_ch = config.layout.channels.len();
        let n_clouds = config.clouds.len();
        println!("✓ Patch is valid. {} cloud(s), {} channel(s).", n_clouds, n_ch);
    } else {
        eprintln!("{} validation error(s):", errors.len());
        for e in &errors {
            eprintln!("  ✗ {}", e);
        }
        std::process::exit(1);
    }
    Ok(())
}

fn cmd_render(patch_path: &str, output_path: &str, duration: f64, emit_hash: bool) -> Result<()> {
    let yaml = std::fs::read_to_string(patch_path)
        .with_context(|| format!("Cannot read '{}'", patch_path))?;
    let config: particelle_schema::ParticelleConfig = serde_yaml::from_str(&yaml)
        .with_context(|| "YAML parse error")?;
    let errors = particelle_schema::validate(&config);
    if !errors.is_empty() {
        for e in &errors {
            eprintln!("  ✗ {}", e);
        }
        anyhow::bail!("Configuration is invalid. Cannot render.");
    }

    let sample_rate = config.engine.sample_rate as f64;
    let block_size = config.engine.block_size;
    let n_channels = config.layout.channels.len();
    let total_frames = (duration * sample_rate) as u64;

    eprintln!(
        "→ Rendering {:.1}s @ {}Hz, {} ch, block {} → '{}'",
        duration, sample_rate as u32, n_channels, block_size, output_path
    );

    let mut writer = particelle_io::AudioFileWriter::create(
        output_path,
        n_channels,
        sample_rate,
        24, // 24-bit output
    ).with_context(|| "Cannot create output file")?;

    // Render loop: generate silence blocks (engine scheduling not yet wired)
    // TODO: connect grain engine here
    let mut frames_rendered = 0u64;
    let mut block = particelle_core::audio_block::AudioBlock::new(n_channels, block_size);

    while frames_rendered < total_frames {
        let remaining = (total_frames - frames_rendered) as usize;
        let frames_this_block = block_size.min(remaining);

        // For now, generate silence — grain engine integration is next
        block.silence();

        // If this is the last block, we need a trimmed block
        if frames_this_block < block_size {
            let mut trimmed = particelle_core::audio_block::AudioBlock::new(n_channels, frames_this_block);
            for ch in 0..n_channels {
                trimmed.channels[ch][..frames_this_block]
                    .copy_from_slice(&block.channels[ch][..frames_this_block]);
            }
            writer.write_block(&trimmed)
                .with_context(|| "Write error")?;
        } else {
            writer.write_block(&block)
                .with_context(|| "Write error")?;
        }

        frames_rendered += frames_this_block as u64;
    }

    let written = writer.finalize()
        .with_context(|| "Finalize error")?;
    eprintln!("✓ Wrote {} frames ({} channels) to '{}'", written, n_channels, output_path);

    if emit_hash {
        use sha2::{Sha256, Digest};
        let file_bytes = std::fs::read(output_path)
            .with_context(|| "Cannot read output for hashing")?;
        let hash = Sha256::digest(&file_bytes);
        println!("SHA-256: {:x}", hash);
    }

    Ok(())
}

fn cmd_run(patch_path: &str) -> Result<()> {
    let yaml = std::fs::read_to_string(patch_path)
        .with_context(|| format!("Cannot read '{}'", patch_path))?;
    let config: particelle_schema::ParticelleConfig = serde_yaml::from_str(&yaml)
        .with_context(|| "YAML parse error")?;
    let errors = particelle_schema::validate(&config);
    if !errors.is_empty() {
        for e in &errors {
            eprintln!("  ✗ {}", e);
        }
        anyhow::bail!("Configuration is invalid. Cannot run.");
    }

    let n_channels = config.layout.channels.len();
    let sample_rate = config.engine.sample_rate as f64;
    let block_size = config.engine.block_size;

    let hw_config = particelle_io::HardwareConfig {
        device_name: config.hardware.as_ref().and_then(|h| h.device_name.clone()),
        n_channels,
        sample_rate,
        block_size,
        ..Default::default()
    };

    let host = particelle_io::HardwareHost::new(hw_config);
    host.run(move |buffer: &mut [f32]| {
        // Zero-fill for now — grain engine callback goes here
        for s in buffer.iter_mut() {
            *s = 0.0;
        }
    }).with_context(|| "Audio stream error")?;

    Ok(())
}

fn cmd_init(channels: usize) -> Result<()> {
    let channel_defs: Vec<String> = if channels == 1 {
        vec!["    - { name: \"M\", azimuth_deg: 0.0, elevation_deg: 0.0 }".into()]
    } else if channels == 2 {
        vec![
            "    - { name: \"L\", azimuth_deg: -30.0, elevation_deg: 0.0 }".into(),
            "    - { name: \"R\", azimuth_deg:  30.0, elevation_deg: 0.0 }".into(),
        ]
    } else {
        // Distribute channels evenly in a circle at ear level
        (0..channels)
            .map(|i| {
                let az = -180.0 + (360.0 * i as f64 / channels as f64);
                format!("    - {{ name: \"CH{}\", azimuth_deg: {:.1}, elevation_deg: 0.0 }}", i + 1, az)
            })
            .collect()
    };

    let yaml = format!(
r#"# Particelle patch — generated by `particelle init -n {channels}`
# Edit this file to configure your grain clouds.
# Documentation: https://github.com/TheColby/Particelle

engine:
  sample_rate: 48000
  block_size: 256

layout:
  channels:
{channel_lines}

tuning:
  mode: twelve_tet

clouds:
  - id: my_cloud
    source: "audio/music_example.wav"
    density: 16.0
    duration: 0.1
    amplitude: 0.5
    position: 0.0
    window:
      type: hann
    listener_pos: {{ x: 0.0, y: 1.0, z: 0.0 }}
    width: 0.5
"#,
        channels = channels,
        channel_lines = channel_defs.join("\n"),
    );
    print!("{}", yaml);
    Ok(())
}

fn cmd_curve(curve_path: &str, resolution: usize) -> Result<()> {
    let json = std::fs::read_to_string(curve_path)
        .with_context(|| format!("Cannot read '{}'", curve_path))?;
    let curve = particelle_curve::CompiledCurve::from_json(&json)
        .with_context(|| "Curve compile error")?;
    let (x_min, x_max) = curve.domain();
    eprintln!("→ Evaluating '{}' over [{:.4}, {:.4}] at {} points", curve_path, x_min, x_max, resolution);
    println!("# x\ty");
    for i in 0..=resolution {
        let t = i as f64 / resolution as f64;
        let x = x_min + t * (x_max - x_min);
        println!("{:.6}\t{:.6}", x, curve.eval(x));
    }
    Ok(())
}

fn cmd_set(patch_path: &str, param: &str, value: &str) -> Result<()> {
    let yaml = std::fs::read_to_string(patch_path)
        .with_context(|| format!("Cannot read '{}'", patch_path))?;

    // Simple key-value replacement in YAML text
    // Find the line containing the param's last segment and replace its value
    let parts: Vec<&str> = param.split('.').collect();
    let key = parts.last().ok_or_else(|| anyhow::anyhow!("Empty parameter path"))?;

    let mut found = false;
    let mut output_lines: Vec<String> = Vec::new();

    for line in yaml.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with(&format!("{}:", key)) || trimmed.starts_with(&format!("{} :", key)) {
            // Replace the value part
            if let Some(colon_pos) = line.find(':') {
                let prefix = &line[..=colon_pos];
                output_lines.push(format!("{} {}", prefix, value));
                found = true;
            } else {
                output_lines.push(line.to_string());
            }
        } else {
            output_lines.push(line.to_string());
        }
    }

    if !found {
        anyhow::bail!("Parameter '{}' not found in '{}'", param, patch_path);
    }

    println!("{}", output_lines.join("\n"));
    Ok(())
}
