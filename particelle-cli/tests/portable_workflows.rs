use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn binary() -> PathBuf {
    if let Some(path) = option_env!("CARGO_BIN_EXE_particelle") {
        return PathBuf::from(path);
    }
    let exe = std::env::current_exe().unwrap();
    exe.parent().unwrap().parent().unwrap().join("particelle")
}

fn run(cwd: &Path, args: &[&str]) {
    let output = Command::new(binary())
        .current_dir(cwd)
        .args(args)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{args:?}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn samples(path: &Path) -> (hound::WavSpec, Vec<f32>) {
    let mut reader = hound::WavReader::open(path).unwrap();
    (
        reader.spec(),
        reader.samples::<f32>().map(Result::unwrap).collect(),
    )
}

#[test]
fn bundle_survives_move_and_stems_are_audible() {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!(
        "particelle-portable-{}-{nonce}",
        std::process::id()
    ));
    let source = root.join("source");
    std::fs::create_dir_all(&source).unwrap();
    let wav = source.join("tone.wav");
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: 48_000,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut writer = hound::WavWriter::create(&wav, spec).unwrap();
    for i in 0..48_000 {
        let value = ((i as f64 * 440.0 * std::f64::consts::TAU / 48_000.0).sin() * 16_000.0) as i16;
        writer.write_sample(value).unwrap();
    }
    writer.finalize().unwrap();
    std::fs::write(source.join("curve.json"), r#"{"segments":[{"x":0,"y":0,"x_end":1,"y_end":0.02,"shape":"linear"}],"extrapolation":{"left":"clamp","right":"clamp"}}"#).unwrap();
    std::fs::copy(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../scales/arabic_24.scl"),
        source.join("scale.scl"),
    )
    .unwrap();
    std::fs::write(
        source.join("map.kbm"),
        "12\n0\n127\n60\n69\n440.0\n24\n0\n1\n2\n3\n4\n5\n6\n7\n8\n9\n10\n11\n",
    )
    .unwrap();
    let patch = source.join("patch.yaml");
    std::fs::write(&patch, r#"engine: {sample_rate: 48000, block_size: 128}
layout:
  channels:
    - {name: FL, azimuth_deg: -30}
    - {name: FR, azimuth_deg: 30}
    - {name: RL, azimuth_deg: -110}
    - {name: RR, azimuth_deg: 110}
tuning: {mode: scala, scl_path: scale.scl, kbm_path: map.kbm}
analysis:
  - {id: level, source: tone.wav, extractor: rms}
clouds:
  - {id: low, source: tone.wav, density: 30, duration: 0.1, position: curve.json, amplitude: 0.7, width: 1, window: {type: hann}}
  - {id: high, source: tone.wav, density: 22, duration: 0.08, position: 0.01, amplitude: 0.5, width: 1, window: {type: hann}}
"#).unwrap();

    let bundle = root.join("bundle");
    run(
        &root,
        &[
            "bundle",
            patch.to_str().unwrap(),
            "-o",
            bundle.to_str().unwrap(),
        ],
    );
    assert!(bundle.join("manifest.json").is_file());
    assert_eq!(std::fs::read_dir(bundle.join("assets")).unwrap().count(), 4);
    run(&root, &["verify-bundle", bundle.to_str().unwrap()]);
    let overwrite = Command::new(binary())
        .current_dir(&root)
        .args([
            "bundle",
            patch.to_str().unwrap(),
            "-o",
            bundle.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(!overwrite.status.success());

    let moved = root.join("moved");
    std::fs::rename(&bundle, &moved).unwrap();
    std::fs::remove_dir_all(&source).unwrap();
    let relocated_patch = moved.join("patch.yaml");
    run(&root, &["verify-bundle", moved.to_str().unwrap()]);
    run(&root, &["validate", relocated_patch.to_str().unwrap()]);
    let mix = root.join("mix.wav");
    let stems = root.join("stems");
    run(
        &root,
        &[
            "render",
            relocated_patch.to_str().unwrap(),
            "-o",
            mix.to_str().unwrap(),
            "--duration",
            "0.25",
            "--stems-dir",
            stems.to_str().unwrap(),
        ],
    );
    let (mix_spec, mix_samples) = samples(&mix);
    assert_eq!(mix_spec.channels, 4);
    assert_eq!(mix_samples.len(), 12_000 * 4);
    let (_, low) = samples(&stems.join("01-low.wav"));
    let (_, high) = samples(&stems.join("02-high.wav"));
    assert!(low.iter().any(|sample| sample.abs() > 0.001));
    assert!(high.iter().any(|sample| sample.abs() > 0.001));
    assert_eq!(low.len(), mix_samples.len());
    assert_eq!(high.len(), mix_samples.len());
    let error = mix_samples
        .iter()
        .zip(low.iter().zip(&high))
        .map(|(mix, (a, b))| (mix - a - b).abs())
        .fold(0.0f32, f32::max);
    assert!(error < 0.0001, "stem sum diverges from mix: {error}");
    let manifest: serde_json::Value =
        serde_json::from_slice(&std::fs::read(stems.join("manifest.json")).unwrap()).unwrap();
    assert_eq!(manifest["channels"], 4);
    assert_eq!(manifest["stems"].as_array().unwrap().len(), 2);
    let reused = Command::new(binary())
        .current_dir(&root)
        .args([
            "render",
            relocated_patch.to_str().unwrap(),
            "-o",
            "reused.wav",
            "--duration",
            "0.25",
            "--stems-dir",
            stems.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(!reused.status.success());
    assert!(!root.join("reused.wav").exists());

    let invalid = Command::new(binary())
        .current_dir(&root)
        .args([
            "render",
            relocated_patch.to_str().unwrap(),
            "-o",
            "invalid.wav",
            "--duration",
            "NaN",
        ])
        .output()
        .unwrap();
    assert!(!invalid.status.success());
    assert!(!root.join("invalid.wav").exists());

    let asset = std::fs::read_dir(moved.join("assets"))
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    std::fs::OpenOptions::new()
        .append(true)
        .open(asset)
        .unwrap()
        .write_all(b"tampered")
        .unwrap();
    let tampered = Command::new(binary())
        .current_dir(&root)
        .args(["verify-bundle", moved.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(!tampered.status.success());
    std::fs::remove_dir_all(root).unwrap();
}
