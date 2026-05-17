use std::path::{Path, PathBuf};
use std::process::Command;

fn bin_path() -> PathBuf {
    if let Ok(path) = std::env::var("CARGO_BIN_EXE_particelle") {
        return PathBuf::from(path);
    }

    let exe = std::env::current_exe().expect("current exe path");
    let debug_dir = exe
        .parent()
        .and_then(|p| p.parent())
        .expect("target/debug directory");
    let mut bin = debug_dir.join("particelle");
    if cfg!(windows) {
        bin.set_extension("exe");
    }
    assert!(
        bin.exists(),
        "unable to locate particelle binary at {}",
        bin.display()
    );
    bin
}

fn test_dir(name: &str) -> PathBuf {
    let base = std::env::var("CARGO_TARGET_TMPDIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| std::env::temp_dir());
    let dir = base.join(format!("particelle_cli_{}_{}", name, std::process::id()));
    std::fs::create_dir_all(&dir).expect("create test directory");
    dir
}

fn run_ok(args: &[&str], cwd: &Path) -> String {
    let output = Command::new(bin_path())
        .args(args)
        .current_dir(cwd)
        .output()
        .expect("spawn particelle");
    assert!(
        output.status.success(),
        "command failed: {:?}\nstdout:\n{}\nstderr:\n{}",
        args,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).expect("utf8 stdout")
}

fn write_init_patch(path: &Path, cwd: &Path) {
    let yaml = run_ok(&["init"], cwd);
    std::fs::write(path, yaml).expect("write init patch");
}

#[test]
fn render_default_is_f32() {
    let cwd = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
    let dir = test_dir("render_default_is_f32");
    let patch = dir.join("patch.yaml");
    let wav = dir.join("default.wav");
    write_init_patch(&patch, &cwd);

    run_ok(
        &[
            "render",
            patch.to_str().expect("patch path utf8"),
            "-o",
            wav.to_str().expect("wav path utf8"),
            "--duration",
            "0.1",
        ],
        &cwd,
    );

    let reader = hound::WavReader::open(&wav).expect("open wav");
    let spec = reader.spec();
    assert_eq!(spec.bits_per_sample, 32);
    assert_eq!(spec.sample_format, hound::SampleFormat::Float);
}

#[test]
fn render_format_pcm24_writes_24bit_integer() {
    let cwd = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
    let dir = test_dir("render_format_pcm24");
    let patch = dir.join("patch.yaml");
    let wav = dir.join("pcm24.wav");
    write_init_patch(&patch, &cwd);

    run_ok(
        &[
            "render",
            patch.to_str().expect("patch path utf8"),
            "-o",
            wav.to_str().expect("wav path utf8"),
            "--duration",
            "0.1",
            "--format",
            "pcm24",
        ],
        &cwd,
    );

    let reader = hound::WavReader::open(&wav).expect("open wav");
    let spec = reader.spec();
    assert_eq!(spec.bits_per_sample, 24);
    assert_eq!(spec.sample_format, hound::SampleFormat::Int);
}

#[test]
fn render_format_pcm16_writes_16bit_integer() {
    let cwd = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
    let dir = test_dir("render_format_pcm16");
    let patch = dir.join("patch.yaml");
    let wav = dir.join("pcm16.wav");
    write_init_patch(&patch, &cwd);

    run_ok(
        &[
            "render",
            patch.to_str().expect("patch path utf8"),
            "-o",
            wav.to_str().expect("wav path utf8"),
            "--duration",
            "0.1",
            "--format",
            "pcm16",
        ],
        &cwd,
    );

    let reader = hound::WavReader::open(&wav).expect("open wav");
    let spec = reader.spec();
    assert_eq!(spec.bits_per_sample, 16);
    assert_eq!(spec.sample_format, hound::SampleFormat::Int);
}

#[test]
fn render_pcm24_flag_forces_pcm24() {
    let cwd = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
    let dir = test_dir("render_pcm24_flag");
    let patch = dir.join("patch.yaml");
    let wav = dir.join("pcm24_flag.wav");
    write_init_patch(&patch, &cwd);

    run_ok(
        &[
            "render",
            patch.to_str().expect("patch path utf8"),
            "-o",
            wav.to_str().expect("wav path utf8"),
            "--duration",
            "0.1",
            "--format",
            "f32",
            "--pcm24",
        ],
        &cwd,
    );

    let reader = hound::WavReader::open(&wav).expect("open wav");
    let spec = reader.spec();
    assert_eq!(spec.bits_per_sample, 24);
    assert_eq!(spec.sample_format, hound::SampleFormat::Int);
}
