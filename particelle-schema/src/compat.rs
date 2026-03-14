use crate::config::ParticelleConfig;

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

fn normalize_signal_expr(value: &mut serde_yaml::Value) {
    let serde_yaml::Value::Mapping(mapping) = value else {
        return;
    };

    if let Some(serde_yaml::Value::String(op)) = mapping_get_mut(mapping, "op") {
        if let Some(stripped) = op.strip_prefix('$') {
            *op = stripped.to_string();
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
        return;
    }

    if let Some(serde_yaml::Value::Sequence(args)) = mapping_get_mut(mapping, "args") {
        for arg in args {
            normalize_signal_expr(arg);
        }
    }
}

fn normalize_window(window: &mut serde_yaml::Value) {
    let serde_yaml::Value::Mapping(mapping) = window else {
        return;
    };

    let Some(kind) = mapping_get_mut(mapping, "type").and_then(|value| string_value(&*value))
    else {
        return;
    };

    match kind {
        "tukey" => {
            mapping
                .entry(value_key("alpha"))
                .or_insert(serde_yaml::Value::Number(serde_yaml::Number::from(0.5)));
        }
        "planck_taper" => {
            mapping
                .entry(value_key("epsilon"))
                .or_insert(serde_yaml::Value::Number(serde_yaml::Number::from(0.1)));
        }
        "dpss" => {
            mapping
                .entry(value_key("half_bandwidth"))
                .or_insert(serde_yaml::Value::Number(serde_yaml::Number::from(4.0)));
        }
        _ => {}
    }
}

fn normalize_clouds(root: &mut serde_yaml::Mapping) {
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
                normalize_signal_expr(expr);
            }
        }

        if let Some(window) = mapping_get_mut(cloud_map, "window") {
            normalize_window(window);
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

fn normalize_tuning(root: &mut serde_yaml::Mapping) {
    let Some(serde_yaml::Value::Mapping(tuning)) = mapping_get_mut(root, "tuning") else {
        return;
    };

    if let Some(serde_yaml::Value::String(mode)) = mapping_get_mut(tuning, "mode") {
        if mode == "just_intonation" {
            *mode = "ji".to_string();
        }
    }

    if !tuning.contains_key(value_key("steps")) {
        if let Some(divisions) = mapping_remove(tuning, "divisions") {
            tuning.insert(value_key("steps"), divisions);
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
                } else {
                    normalized.push(ratio.clone());
                }
            }
            _ => normalized.push(ratio.clone()),
        }
    }

    *ratios = normalized;
}

pub fn normalize_yaml_value(value: &mut serde_yaml::Value) {
    let serde_yaml::Value::Mapping(root) = value else {
        return;
    };

    normalize_tuning(root);
    normalize_clouds(root);
}

pub fn parse_yaml_compat(yaml: &str) -> Result<ParticelleConfig, serde_yaml::Error> {
    let mut value: serde_yaml::Value = serde_yaml::from_str(yaml)?;
    normalize_yaml_value(&mut value);
    serde_yaml::from_value(value)
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

        let config = parse_yaml_compat(yaml).unwrap();
        assert!(matches!(
            config.tuning,
            crate::config::TuningConfig::Edo { steps: 31 }
        ));
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

        let config = parse_yaml_compat(yaml).unwrap();
        let crate::config::TuningConfig::Ji { ratios } = config.tuning else {
            panic!("expected JI tuning");
        };
        assert_eq!(ratios.len(), 3);
        assert_eq!(ratios[1].degree, 1);
        assert_eq!(ratios[1].num, 9);
        assert_eq!(ratios[1].den, 8);
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

        let config = parse_yaml_compat(yaml).unwrap();
        let cloud = &config.clouds[0];
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
    }
}
