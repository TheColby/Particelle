use crate::config::{ParticelleConfig, CURRENT_SCHEMA_VERSION};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MigrationNote {
    pub id: &'static str,
    pub description: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MigrationReport {
    pub source_schema_version: u32,
    pub target_schema_version: u32,
    pub notes: Vec<MigrationNote>,
}

#[derive(Debug, Clone)]
pub struct CompatParseOutput {
    pub config: ParticelleConfig,
    pub report: MigrationReport,
}

fn value_key(name: &str) -> serde_yaml::Value {
    serde_yaml::Value::String(name.to_string())
}

fn mapping_get_mut<'a>(
    mapping: &'a mut serde_yaml::Mapping,
    key: &str,
) -> Option<&'a mut serde_yaml::Value> {
    mapping.get_mut(value_key(key))
}

fn mapping_remove(mapping: &mut serde_yaml::Mapping, key: &str) -> Option<serde_yaml::Value> {
    mapping.remove(value_key(key))
}

fn string_value(value: &serde_yaml::Value) -> Option<&str> {
    match value {
        serde_yaml::Value::String(text) => Some(text.as_str()),
        _ => None,
    }
}

fn push_note(report: &mut MigrationReport, id: &'static str, description: &'static str) {
    if report.notes.iter().any(|note| note.id == id) {
        return;
    }
    report.notes.push(MigrationNote { id, description });
}

fn detect_source_schema_version(value: &serde_yaml::Value) -> u32 {
    let serde_yaml::Value::Mapping(root) = value else {
        return 0;
    };
    let Some(schema_version_value) = root.get(value_key("schema_version")) else {
        return 0;
    };

    match schema_version_value {
        serde_yaml::Value::Number(number) => number.as_u64().map(|v| v as u32).unwrap_or(0),
        _ => 0,
    }
}

fn normalize_schema_version(root: &mut serde_yaml::Mapping, report: &mut MigrationReport) {
    let current = serde_yaml::to_value(CURRENT_SCHEMA_VERSION).unwrap();
    let source = report.source_schema_version;

    if source == 0 {
        root.insert(value_key("schema_version"), current);
        push_note(
            report,
            "schema.version.introduced",
            "Inserted schema_version for legacy patch without an explicit version.",
        );
        return;
    }

    if source < CURRENT_SCHEMA_VERSION {
        root.insert(value_key("schema_version"), current);
        push_note(
            report,
            "schema.version.bumped",
            "Upgraded schema_version to the current canonical version.",
        );
    }
}

fn normalize_signal_expr(value: &mut serde_yaml::Value, report: &mut MigrationReport) {
    let serde_yaml::Value::Mapping(mapping) = value else {
        return;
    };

    if let Some(serde_yaml::Value::String(op)) = mapping_get_mut(mapping, "op") {
        if let Some(stripped) = op.strip_prefix('$') {
            *op = stripped.to_string();
            push_note(
                report,
                "signal.op.strip_dollar_prefix",
                "Normalized signal op aliases from '$name' to 'name'.",
            );
        }
    }

    let is_curve = mapping_get_mut(mapping, "op")
        .and_then(|value| string_value(&*value))
        .is_some_and(|op| op == "curve");

    let curve_ref = if is_curve {
        mapping_remove(mapping, "ref").and_then(|ref_value| match ref_value {
            serde_yaml::Value::String(path) => Some(path),
            _ => None,
        })
    } else {
        None
    };

    if let Some(path) = curve_ref {
        *value = serde_yaml::Value::String(path);
        push_note(
            report,
            "signal.curve.ref_to_string",
            "Normalized curve op nodes into canonical string reference form.",
        );
        return;
    }

    if let Some(serde_yaml::Value::Sequence(args)) = mapping_get_mut(mapping, "args") {
        for arg in args {
            normalize_signal_expr(arg, report);
        }
    }
}

fn normalize_window(window: &mut serde_yaml::Value, report: &mut MigrationReport) {
    let serde_yaml::Value::Mapping(mapping) = window else {
        return;
    };

    let Some(kind) = mapping_get_mut(mapping, "type").and_then(|value| string_value(&*value))
    else {
        return;
    };

    match kind {
        "tukey" => {
            if !mapping.contains_key(value_key("alpha")) {
                mapping.insert(
                    value_key("alpha"),
                    serde_yaml::Value::Number(serde_yaml::Number::from(0.5)),
                );
                push_note(
                    report,
                    "window.defaults.tukey_alpha",
                    "Added default alpha=0.5 for legacy tukey window nodes missing alpha.",
                );
            }
        }
        "planck_taper" => {
            if !mapping.contains_key(value_key("epsilon")) {
                mapping.insert(
                    value_key("epsilon"),
                    serde_yaml::Value::Number(serde_yaml::Number::from(0.1)),
                );
                push_note(
                    report,
                    "window.defaults.planck_epsilon",
                    "Added default epsilon=0.1 for legacy planck_taper window nodes missing epsilon.",
                );
            }
        }
        "dpss" => {
            if !mapping.contains_key(value_key("half_bandwidth")) {
                mapping.insert(
                    value_key("half_bandwidth"),
                    serde_yaml::Value::Number(serde_yaml::Number::from(4.0)),
                );
                push_note(
                    report,
                    "window.defaults.dpss_half_bandwidth",
                    "Added default half_bandwidth=4.0 for legacy dpss window nodes missing half_bandwidth.",
                );
            }
        }
        _ => {}
    }
}

fn normalize_clouds(root: &mut serde_yaml::Mapping, report: &mut MigrationReport) {
    let Some(serde_yaml::Value::Sequence(clouds)) = mapping_get_mut(root, "clouds") else {
        return;
    };

    for cloud in clouds {
        let serde_yaml::Value::Mapping(cloud_map) = cloud else {
            continue;
        };

        for field in [
            "density",
            "duration",
            "position",
            "amplitude",
            "width",
            "directivity",
            "orientation_azimuth",
            "orientation_elevation",
        ] {
            if let Some(expr) = mapping_get_mut(cloud_map, field) {
                normalize_signal_expr(expr, report);
            }
        }

        if let Some(window) = mapping_get_mut(cloud_map, "window") {
            normalize_window(window, report);
        }
    }
}

fn parse_legacy_ratio(text: &str) -> Option<(u64, u64)> {
    let trimmed = text.trim();
    if let Some((num, den)) = trimmed.split_once('/') {
        let num = num.trim().parse().ok()?;
        let den = den.trim().parse().ok()?;
        return Some((num, den));
    }

    let num = trimmed.parse().ok()?;
    Some((num, 1))
}

fn normalize_tuning(root: &mut serde_yaml::Mapping, report: &mut MigrationReport) {
    let Some(serde_yaml::Value::Mapping(tuning)) = mapping_get_mut(root, "tuning") else {
        return;
    };

    if let Some(serde_yaml::Value::String(mode)) = mapping_get_mut(tuning, "mode") {
        if mode == "just_intonation" {
            *mode = "ji".to_string();
            push_note(
                report,
                "tuning.mode.just_intonation_to_ji",
                "Renamed tuning.mode from just_intonation to ji.",
            );
        }
    }

    if !tuning.contains_key(value_key("steps")) {
        if let Some(divisions) = mapping_remove(tuning, "divisions") {
            tuning.insert(value_key("steps"), divisions);
            push_note(
                report,
                "tuning.edo.divisions_to_steps",
                "Renamed tuning.divisions to tuning.steps for EDO mode.",
            );
        }
    }

    let ratios = match mapping_get_mut(tuning, "ratios") {
        Some(serde_yaml::Value::Sequence(ratios)) => ratios,
        _ => return,
    };

    if ratios
        .iter()
        .all(|value| matches!(value, serde_yaml::Value::Mapping(_)))
    {
        return;
    }

    let mut normalized = Vec::with_capacity(ratios.len());
    let mut changed = false;
    for (degree, ratio) in ratios.iter().enumerate() {
        match ratio {
            serde_yaml::Value::String(text) => {
                if let Some((num, den)) = parse_legacy_ratio(text) {
                    let mut item = serde_yaml::Mapping::new();
                    item.insert(
                        value_key("degree"),
                        serde_yaml::to_value(degree as u32).unwrap(),
                    );
                    item.insert(value_key("num"), serde_yaml::to_value(num).unwrap());
                    item.insert(value_key("den"), serde_yaml::to_value(den).unwrap());
                    normalized.push(serde_yaml::Value::Mapping(item));
                    changed = true;
                } else {
                    normalized.push(ratio.clone());
                }
            }
            serde_yaml::Value::Number(number) => {
                if let Some(num) = number.as_u64() {
                    let mut item = serde_yaml::Mapping::new();
                    item.insert(
                        value_key("degree"),
                        serde_yaml::to_value(degree as u32).unwrap(),
                    );
                    item.insert(value_key("num"), serde_yaml::to_value(num).unwrap());
                    item.insert(value_key("den"), serde_yaml::to_value(1_u64).unwrap());
                    normalized.push(serde_yaml::Value::Mapping(item));
                    changed = true;
                } else {
                    normalized.push(ratio.clone());
                }
            }
            _ => normalized.push(ratio.clone()),
        }
    }

    if changed {
        push_note(
            report,
            "tuning.ji.ratio_list_to_structs",
            "Converted legacy JI ratio list entries into structured degree/num/den mappings.",
        );
    }

    *ratios = normalized;
}

fn normalize_yaml_value_with_report(value: &mut serde_yaml::Value, report: &mut MigrationReport) {
    let serde_yaml::Value::Mapping(root) = value else {
        return;
    };

    normalize_schema_version(root, report);
    normalize_tuning(root, report);
    normalize_clouds(root, report);
}

pub fn normalize_yaml_value(value: &mut serde_yaml::Value) {
    let source_version = detect_source_schema_version(value);
    let mut report = MigrationReport {
        source_schema_version: source_version,
        target_schema_version: source_version,
        notes: Vec::new(),
    };
    normalize_yaml_value_with_report(value, &mut report);
}

pub fn parse_yaml_compat_with_report(yaml: &str) -> Result<CompatParseOutput, serde_yaml::Error> {
    let mut value: serde_yaml::Value = serde_yaml::from_str(yaml)?;
    let source_version = detect_source_schema_version(&value);
    let mut report = MigrationReport {
        source_schema_version: source_version,
        target_schema_version: source_version,
        notes: Vec::new(),
    };

    normalize_yaml_value_with_report(&mut value, &mut report);
    let config: ParticelleConfig = serde_yaml::from_value(value)?;
    report.target_schema_version = config.schema_version;

    Ok(CompatParseOutput { config, report })
}

pub fn parse_yaml_compat(yaml: &str) -> Result<ParticelleConfig, serde_yaml::Error> {
    Ok(parse_yaml_compat_with_report(yaml)?.config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_legacy_edo_divisions() {
        let yaml = r#"
engine: { sample_rate: 48000, block_size: 256 }
layout:
  channels:
    - { name: "L", azimuth_deg: -30.0 }
    - { name: "R", azimuth_deg: 30.0 }
tuning:
  mode: edo
  divisions: 31
"#;

        let output = parse_yaml_compat_with_report(yaml).unwrap();
        assert!(matches!(
            output.config.tuning,
            crate::config::TuningConfig::Edo { steps: 31 }
        ));
        assert_eq!(output.report.source_schema_version, 0);
        assert_eq!(output.report.target_schema_version, CURRENT_SCHEMA_VERSION);
        assert!(output
            .report
            .notes
            .iter()
            .any(|n| n.id == "tuning.edo.divisions_to_steps"));
    }

    #[test]
    fn parses_legacy_just_intonation_ratios() {
        let yaml = r#"
engine: { sample_rate: 48000, block_size: 256 }
layout:
  channels:
    - { name: "L", azimuth_deg: -30.0 }
    - { name: "R", azimuth_deg: 30.0 }
tuning:
  mode: just_intonation
  ratios: ["1/1", "9/8", "5/4"]
"#;

        let output = parse_yaml_compat_with_report(yaml).unwrap();
        let crate::config::TuningConfig::Ji { ratios } = output.config.tuning else {
            panic!("expected JI tuning");
        };
        assert_eq!(ratios.len(), 3);
        assert_eq!(ratios[1].degree, 1);
        assert_eq!(ratios[1].num, 9);
        assert_eq!(ratios[1].den, 8);
        assert!(output
            .report
            .notes
            .iter()
            .any(|n| n.id == "tuning.mode.just_intonation_to_ji"));
    }

    #[test]
    fn parses_legacy_curve_signal_and_osc_alias() {
        let yaml = r#"
engine: { sample_rate: 48000, block_size: 256 }
layout:
  channels:
    - { name: "L", azimuth_deg: -30.0 }
clouds:
  - id: test
    source: audio/music_example.wav
    density: 10.0
    duration: 0.1
    position:
      op: curve
      ref: curves/stretch_pos.json
    amplitude:
      op: "$osc"
      args: [triangle, 0.1]
    width: 0.5
    window: { type: tukey }
"#;

        let output = parse_yaml_compat_with_report(yaml).unwrap();
        let cloud = &output.config.clouds[0];
        assert!(matches!(
            cloud.position,
            crate::config::SignalExprConfig::Ref(ref path) if path == "curves/stretch_pos.json"
        ));

        let crate::config::SignalExprConfig::Expr(op) = &cloud.amplitude else {
            panic!("expected normalized amplitude expr");
        };
        assert_eq!(op.op, "osc");
        assert_eq!(cloud.window.kind, "tukey");
        assert!(cloud.window.params.contains_key("alpha"));
        assert!(output
            .report
            .notes
            .iter()
            .any(|n| n.id == "signal.op.strip_dollar_prefix"));
    }

    #[test]
    fn injects_current_schema_version_for_legacy_patch() {
        let yaml = r#"
engine: { sample_rate: 48000, block_size: 256 }
layout:
  channels:
    - { name: "L", azimuth_deg: -30.0 }
"#;

        let output = parse_yaml_compat_with_report(yaml).unwrap();
        assert_eq!(output.report.source_schema_version, 0);
        assert_eq!(output.config.schema_version, CURRENT_SCHEMA_VERSION);
        assert!(output
            .report
            .notes
            .iter()
            .any(|n| n.id == "schema.version.introduced"));
    }

    #[test]
    fn preserves_explicit_future_schema_version() {
        let yaml = r#"
schema_version: 999
engine: { sample_rate: 48000, block_size: 256 }
layout:
  channels:
    - { name: "L", azimuth_deg: -30.0 }
"#;

        let output = parse_yaml_compat_with_report(yaml).unwrap();
        assert_eq!(output.report.source_schema_version, 999);
        assert_eq!(output.config.schema_version, 999);
        assert!(output.report.notes.is_empty());
    }
}
