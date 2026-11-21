use anyhow::{bail, Context, Result};
use particelle_schema::ParticelleConfig;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs::{self, File};
use std::io::Read;
use std::path::{Component, Path, PathBuf};

use crate::patch;

fn fingerprint(path: &Path) -> Result<(String, u64)> {
    let mut file = File::open(path).with_context(|| format!("Cannot read '{}'", path.display()))?;
    let mut digest = Sha256::new();
    let mut bytes = 0u64;
    let mut buffer = [0u8; 64 * 1024];
    loop {
        let n = file.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        digest.update(&buffer[..n]);
        bytes += n as u64;
    }
    Ok((format!("{:x}", digest.finalize()), bytes))
}

fn asset_name(path: &Path, digest: &str) -> String {
    let extension = path
        .extension()
        .and_then(|value| value.to_str())
        .filter(|value| value.len() <= 8 && value.chars().all(|ch| ch.is_ascii_alphanumeric()))
        .unwrap_or("bin")
        .to_ascii_lowercase();
    format!("assets/{digest}.{extension}")
}

pub fn create(mut config: ParticelleConfig, output: &Path) -> Result<()> {
    if config.clouds.iter().any(|cloud| cloud.source == "input") {
        bail!("Cannot bundle a live 'input' source; use an audio file instead");
    }
    let errors = particelle_schema::validate(&config);
    if !errors.is_empty() {
        bail!("Cannot bundle invalid patch: {}", errors[0]);
    }

    let mut assets: BTreeMap<PathBuf, (String, String, u64)> = BTreeMap::new();
    patch::visit_assets(&mut config, &mut |path, _field| {
        let source = PathBuf::from(&*path);
        let name = if let Some(cached) = assets.get(&source) {
            cached.0.clone()
        } else {
            let (digest, bytes) = fingerprint(&source)?;
            let name = asset_name(&source, &digest);
            assets.insert(source.clone(), (name.clone(), digest.clone(), bytes));
            name
        };
        *path = name;
        Ok(())
    })?;

    let yaml = serde_yaml::to_string(&config)?;
    fs::create_dir(output).with_context(|| {
        format!(
            "Cannot create bundle '{}'; choose a new directory",
            output.display()
        )
    })?;
    let marker = output.join("INCOMPLETE");
    fs::write(
        &marker,
        "Bundle creation did not finish. Do not use this directory.\n",
    )?;
    fs::create_dir(output.join("assets"))?;
    let mut entries = BTreeMap::new();
    for (source, (name, expected, bytes)) in assets {
        let target = output.join(&name);
        if !target.exists() {
            fs::copy(&source, &target)
                .with_context(|| format!("Cannot copy '{}'", source.display()))?;
        }
        let (actual, copied_bytes) = fingerprint(&target)?;
        if actual != expected || copied_bytes != bytes {
            bail!("Asset changed while bundling: '{}'", source.display());
        }
        entries.insert(
            name,
            serde_json::json!({"sha256": actual, "bytes": copied_bytes}),
        );
    }
    fs::write(output.join("patch.yaml"), &yaml)?;
    let (patch_digest, patch_bytes) = fingerprint(&output.join("patch.yaml"))?;
    entries.insert(
        "patch.yaml".to_string(),
        serde_json::json!({"sha256": patch_digest, "bytes": patch_bytes}),
    );
    let manifest = serde_json::json!({
        "schema_version": 1,
        "particelle_version": env!("CARGO_PKG_VERSION"),
        "files": entries,
    });
    fs::write(
        output.join("manifest.json"),
        format!("{}\n", serde_json::to_string_pretty(&manifest)?),
    )?;
    fs::remove_file(marker)?;
    println!("Created portable patch bundle: {}", output.display());
    Ok(())
}

pub fn verify(directory: &Path) -> Result<()> {
    anyhow::ensure!(
        !directory.join("INCOMPLETE").exists(),
        "Bundle is marked INCOMPLETE"
    );
    let manifest: serde_json::Value = serde_json::from_slice(
        &fs::read(directory.join("manifest.json")).context("Cannot read bundle manifest")?,
    )?;
    anyhow::ensure!(
        manifest["schema_version"] == 1,
        "Unsupported bundle manifest version"
    );
    let files = manifest["files"]
        .as_object()
        .context("Bundle manifest has no file list")?;
    anyhow::ensure!(
        files.contains_key("patch.yaml"),
        "Bundle manifest omits patch.yaml"
    );
    for (name, entry) in files {
        let relative = Path::new(name);
        anyhow::ensure!(
            relative
                .components()
                .all(|part| matches!(part, Component::Normal(_))),
            "Unsafe bundle path '{name}'"
        );
        let target = directory.join(relative);
        anyhow::ensure!(
            fs::symlink_metadata(&target)
                .with_context(|| format!("Missing bundle file '{name}'"))?
                .file_type()
                .is_file(),
            "Bundle file '{name}' is not a regular file"
        );
        let (actual, bytes) = fingerprint(&target)?;
        anyhow::ensure!(
            entry["sha256"].as_str() == Some(actual.as_str())
                && entry["bytes"].as_u64() == Some(bytes),
            "Bundle file '{name}' does not match its manifest"
        );
    }
    let patch = directory.join("patch.yaml");
    let mut portable = crate::parse_patch_config(&fs::read_to_string(&patch)?)?;
    anyhow::ensure!(
        portable.clouds.iter().all(|cloud| cloud.source != "input"),
        "Bundle patch contains a live input source"
    );
    patch::visit_assets(&mut portable, &mut |path, field| {
        let relative = Path::new(path);
        anyhow::ensure!(
            relative
                .components()
                .all(|part| matches!(part, Component::Normal(_)))
                && files.contains_key(path),
            "{field}: reference '{path}' is not a bundled file"
        );
        Ok(())
    })?;
    let config = crate::load_patch_config(&patch.to_string_lossy())?;
    let errors = particelle_schema::validate(&config);
    anyhow::ensure!(errors.is_empty(), "Bundled patch is invalid: {}", errors[0]);
    println!("Verified portable patch bundle: {}", directory.display());
    Ok(())
}
