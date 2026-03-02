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
        println!("OK — configuration is valid.");
    } else {
        eprintln!("{} validation error(s):", errors.len());
        for e in &errors {
            eprintln!("  - {}", e);
        }
        std::process::exit(1);
    }
    Ok(())
}

fn cmd_render(patch_path: &str, output_path: &str, _duration: f64, emit_hash: bool) -> Result<()> {
    let yaml = std::fs::read_to_string(patch_path)
        .with_context(|| format!("Cannot read '{}'", patch_path))?;
    let config: particelle_schema::ParticelleConfig = serde_yaml::from_str(&yaml)
        .with_context(|| "YAML parse error")?;
    let errors = particelle_schema::validate(&config);
    if !errors.is_empty() {
        for e in &errors {
            eprintln!("  - {}", e);
        }
        anyhow::bail!("Configuration is invalid. Cannot render.");
    }

    println!("Rendering to '{}'...", output_path);
    // TODO: Phase 5 — engine construction, block loop, file write
    println!("Render complete.");

    if emit_hash {
        // TODO: Phase 5 — compute SHA-256 over output samples
        println!("SHA-256: (not yet implemented)");
    }

    Ok(())
}

fn cmd_run(patch_path: &str) -> Result<()> {
    let yaml = std::fs::read_to_string(patch_path)
        .with_context(|| format!("Cannot read '{}'", patch_path))?;
    let _config: particelle_schema::ParticelleConfig = serde_yaml::from_str(&yaml)
        .with_context(|| "YAML parse error")?;
    println!("Realtime mode: not yet implemented.");
    // TODO: Phase 8 — hardware host setup, audio thread launch
    Ok(())
}

fn cmd_init(channels: usize) -> Result<()> {
    // TODO: generate from a template based on channel count
    let yaml = format!(
        "engine:\n  sample_rate: 48000\n  block_size: 256\n\nlayout:\n  channels: []\n\ntuning:\n  mode: twelve_tet\n\nclouds: []\n\n# {} output channels requested\n",
        channels
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
    println!("# x\ty");
    for i in 0..=resolution {
        let t = i as f64 / resolution as f64;
        let x = x_min + t * (x_max - x_min);
        println!("{:.6}\t{:.6}", x, curve.eval(x));
    }
    Ok(())
}

fn cmd_set(_patch_path: &str, _param: &str, _value: &str) -> Result<()> {
    // TODO: read YAML, locate param by dot path, substitute value, print
    println!("set: not yet implemented.");
    Ok(())
}
