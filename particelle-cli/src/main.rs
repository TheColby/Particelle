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
    about = "A research-grade granular synthesis engine",
    long_about = None,
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Validate a YAML patch file and report any errors.
    Validate {
        /// Path to the YAML configuration file.
        patch: String,
    },

    /// Render a patch to an audio file offline.
    Render {
        /// Path to the YAML configuration file.
        patch: String,

        /// Output audio file path.
        #[arg(short, long)]
        output: String,

        /// Render duration in seconds.
        #[arg(short, long, default_value = "10.0")]
        duration: f64,

        /// Print deterministic SHA-256 hash of output after rendering.
        #[arg(long)]
        hash: bool,
    },

    /// Run a patch in realtime using the configured hardware device.
    Run {
        /// Path to the YAML configuration file.
        patch: String,
    },

    /// Generate a default YAML patch to stdout.
    Init {
        /// Number of output channels.
        #[arg(short = 'n', long, default_value = "2")]
        channels: usize,
    },

    /// Inspect and print a JSON curve file.
    Curve {
        /// Path to the JSON curve file.
        curve: String,

        /// Number of evaluation points to print.
        #[arg(short, long, default_value = "64")]
        resolution: usize,
    },

    /// Override a single parameter value in a patch (prints modified YAML to stdout).
    Set {
        /// Path to the YAML configuration file.
        patch: String,

        /// Canonical parameter path (e.g., `engine.sample_rate`).
        param: String,

        /// New value.
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
