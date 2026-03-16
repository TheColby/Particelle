# Schema Versioning and Migration Metadata

Particelle patches now carry an explicit `schema_version` field.

- Current schema version: `2`
- Legacy patches without `schema_version` are treated as version `0` and normalized forward.
- Future versions (`schema_version > 2`) are parsed but rejected by validation until support is added.

## Patch Header

```yaml
schema_version: 2
engine:
  sample_rate: 48000
  block_size: 256
```

## Migration Notes

The compatibility layer emits migration notes with stable IDs so migrations are auditable across releases.

| Migration ID | Description |
| --- | --- |
| `schema.version.introduced` | Inserted `schema_version` for legacy patches without an explicit version. |
| `schema.version.bumped` | Upgraded older declared schema versions to the current canonical version. |
| `tuning.mode.just_intonation_to_ji` | Renamed `tuning.mode: just_intonation` to `tuning.mode: ji`. |
| `tuning.edo.divisions_to_steps` | Renamed `tuning.divisions` to `tuning.steps`. |
| `tuning.ji.ratio_list_to_structs` | Converted legacy JI ratio list entries into structured `degree/num/den`. |
| `signal.op.strip_dollar_prefix` | Normalized signal operator aliases from `$name` to `name`. |
| `signal.curve.ref_to_string` | Normalized `op: curve` + `ref` nodes into canonical string signal refs. |
| `window.defaults.tukey_alpha` | Added `alpha=0.5` when legacy `tukey` nodes omit it. |
| `window.defaults.planck_epsilon` | Added `epsilon=0.1` when legacy `planck_taper` nodes omit it. |
| `window.defaults.dpss_half_bandwidth` | Added `half_bandwidth=4.0` when legacy `dpss` nodes omit it. |

## CLI Audit Path

`particelle validate patch.yaml` prints the applied migration notes (if any), plus the final schema version used for validation.
