use anyhow::{bail, Context, Result};
use particelle_schema::{ParticelleConfig, SignalExprConfig, TuningConfig};
use std::path::{Path, PathBuf};

fn asset(path: &mut String, base: &Path, field: &str) -> Result<()> {
    let original = PathBuf::from(&*path);
    let preferred = base.join(&original);
    let resolved = if preferred.is_file() {
        preferred
    } else if !original.is_absolute() && original.is_file() {
        eprintln!(
            "warning: {field}: patch-relative asset '{}' missing; using cwd fallback '{}'",
            preferred.display(),
            original.display()
        );
        original
    } else {
        bail!(
            "{field}: missing asset '{}' (patch-relative '{}'; cwd fallback '{}')",
            path,
            preferred.display(),
            original.display()
        );
    };
    *path = resolved.canonicalize()?.to_string_lossy().into_owned();
    Ok(())
}

fn signal(expr: &mut SignalExprConfig, base: &Path, field: &str) -> Result<()> {
    match expr {
        SignalExprConfig::Ref(path) if !path.starts_with('$') => asset(path, base, field)?,
        SignalExprConfig::Expr(op) => {
            for (i, arg) in op.args.iter_mut().enumerate() {
                // These operator arguments are symbols, not curve filenames.
                if matches!(
                    (op.op.as_str(), i),
                    ("osc", 0) | ("map", 1) | ("lorenz", 3) | ("rossler", 3) | ("henon", 2)
                ) {
                    continue;
                }
                signal(arg, base, &format!("{field}.args[{i}]"))?;
            }
        }
        _ => {}
    }
    Ok(())
}

pub fn resolve(config: &mut ParticelleConfig, patch: &str) -> Result<()> {
    let cwd = std::env::current_dir()?;
    let base = if patch == "-" {
        cwd.clone()
    } else {
        cwd.join(patch).parent().unwrap_or(&cwd).to_path_buf()
    };
    for (i, cloud) in config.clouds.iter_mut().enumerate() {
        if cloud.source != "input" {
            asset(&mut cloud.source, &base, &format!("clouds[{i}].source"))?;
        }
        for (name, expr) in [
            ("density", &mut cloud.density),
            ("duration", &mut cloud.duration),
            ("position", &mut cloud.position),
            ("amplitude", &mut cloud.amplitude),
            ("width", &mut cloud.width),
            ("directivity", &mut cloud.directivity),
            ("orientation_azimuth", &mut cloud.orientation_azimuth),
            ("orientation_elevation", &mut cloud.orientation_elevation),
            ("playback_rate", &mut cloud.playback_rate),
        ] {
            signal(expr, &base, &format!("clouds[{i}].{name}"))?;
        }
    }
    for (i, analysis) in config.analysis.iter_mut().enumerate() {
        asset(
            &mut analysis.source,
            &base,
            &format!("analysis[{i}].source"),
        )?;
    }
    if let TuningConfig::Scala { scl_path, kbm_path } = &mut config.tuning {
        asset(scl_path, &base, "tuning.scl_path")?;
        if let Some(path) = kbm_path {
            asset(path, &base, "tuning.kbm_path")?;
        }
    }
    for (i, binding) in config.routing.midi_bindings.iter_mut().enumerate() {
        if let Some(expr) = &mut binding.transform {
            signal(
                expr,
                &base,
                &format!("routing.midi_bindings[{i}].transform"),
            )?;
        }
    }
    Ok(())
}

/// Update exactly one existing node. Both clouds.0.width and clouds[0].width
/// are accepted; invalid indices never grow or partially modify the document.
pub fn set(yaml: &str, path: &str, replacement: &str) -> Result<String> {
    let mut root: serde_yaml::Value = serde_yaml::from_str(yaml)?;
    let normalized = path.replace('[', ".").replace(']', "");
    let canonical = normalized.split('.').collect::<Vec<_>>().join(".");
    if path.is_empty()
        || path.contains("[]")
        || path.matches('[').count() != path.matches(']').count()
        || path.contains("[[")
        || path.contains("][")
        || path.contains(']') && !path.contains('[')
        || canonical.starts_with('.')
        || canonical.ends_with('.')
    {
        bail!("Invalid parameter path '{path}'");
    }
    // Bracket syntax is only for numeric sequence indices, never mapping keys.
    for segment in path.split('[').skip(1) {
        let Some((index, rest)) = segment.split_once(']') else {
            bail!("Invalid parameter path '{path}'");
        };
        if index.parse::<usize>().is_err() || !matches!(rest.chars().next(), None | Some('.' | '['))
        {
            bail!("Invalid parameter path '{path}'");
        }
    }
    let mut node = &mut root;
    for part in normalized.split('.') {
        if part.is_empty() {
            bail!("Invalid parameter path '{path}'");
        }
        node = match node {
            serde_yaml::Value::Mapping(map) => map
                .get_mut(serde_yaml::Value::String(part.into()))
                .with_context(|| format!("Parameter '{path}': missing key '{part}'"))?,
            serde_yaml::Value::Sequence(seq) => {
                let index: usize = part.parse().with_context(|| {
                    format!("Parameter '{path}': expected array index, got '{part}'")
                })?;
                seq.get_mut(index).with_context(|| {
                    format!("Parameter '{path}': array index {index} out of bounds")
                })?
            }
            _ => bail!("Parameter '{path}': cannot traverse scalar at '{part}'"),
        };
    }
    *node = serde_yaml::from_str(replacement).context("Invalid replacement YAML")?;
    Ok(serde_yaml::to_string(&root)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_changes_only_selected_node() {
        let yaml = "engine: {sample_rate: 48000}\nclouds:\n  - {density: 1}\n  - {density: 2}\n";
        let changed = set(yaml, "clouds[1].density", "7").unwrap();
        let value: serde_yaml::Value = serde_yaml::from_str(&changed).unwrap();
        assert_eq!(value["clouds"][0]["density"].as_i64(), Some(1));
        assert_eq!(value["clouds"][1]["density"].as_i64(), Some(7));
        assert!(set(yaml, "clouds[2].density", "7").is_err());
        assert!(set(yaml, "clouds[abc].density", "7").is_err());
        assert!(set(yaml, "clouds[0]density", "7").is_err());
    }

    #[test]
    fn resolves_patch_relative_assets() {
        let dir = std::env::temp_dir().join(format!("particelle-patch-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let source = dir.join("sample.wav");
        std::fs::write(&source, b"test").unwrap();
        let curves = dir.join("curves");
        std::fs::create_dir_all(&curves).unwrap();
        let curve = curves.join("control.json");
        std::fs::write(&curve, b"{}").unwrap();
        let patch = dir.join("patch.yaml");
        let yaml = "engine: {sample_rate: 48000, block_size: 64}\nlayout: {channels: [{name: L, azimuth_deg: 0}]}\nclouds:\n  - id: c\n    source: sample.wav\n    density: {op: add, args: [curves/control.json, '$density_mod']}\n    duration: 1\n    position: 0\n    amplitude: 1\n    window: {type: hann}\n    width: 1\n";
        let mut config: ParticelleConfig = serde_yaml::from_str(yaml).unwrap();
        resolve(&mut config, patch.to_str().unwrap()).unwrap();
        assert_eq!(
            config.clouds[0].source,
            source.canonicalize().unwrap().to_string_lossy()
        );
        let SignalExprConfig::Expr(op) = &config.clouds[0].density else {
            panic!("expected expression");
        };
        assert!(
            matches!(&op.args[0], SignalExprConfig::Ref(path) if path == &curve.canonicalize().unwrap().to_string_lossy())
        );
        assert!(matches!(&op.args[1], SignalExprConfig::Ref(path) if path == "$density_mod"));
        std::fs::remove_file(source).unwrap();
        std::fs::remove_file(curve).unwrap();
        std::fs::remove_dir(curves).unwrap();
        std::fs::remove_dir(dir).unwrap();
    }
}
