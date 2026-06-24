# Delta for project-file-management

## ADDED Requirements

### Requirement: Push State Fields in Metadata

The `Metadata` struct in the project's `.config/metadata.json` MUST include:

| Field | Type | Default | Purpose |
|-------|------|---------|---------|
| `push_enabled` | `bool` | `false` | Whether auto-push to remote is active for this project |
| `consecutive_failures` | `u32` | `0` | Count of consecutive push failures for 3-strike rule |

Both fields SHALL use serde `#[serde(default)]` so projects created before this change open without error. Existing `metadata.json` files lacking these fields SHALL deserialize with defaults (`push_enabled: false`, `consecutive_failures: 0`).

#### Scenario: New project gets push defaults
- GIVEN a project is created with `crear_proyecto`
- WHEN `metadata.json` is written
- THEN it SHALL contain `"push_enabled": false` and `"consecutive_failures": 0`

#### Scenario: Old project without push fields loads successfully
- GIVEN a `metadata.json` from a pre-existing project without `push_enabled` or `consecutive_failures` keys
- WHEN the metadata is deserialized
- THEN the struct SHALL default to `push_enabled: false` and `consecutive_failures: 0`
- AND no error SHALL occur

#### Scenario: Push fields survive metadata round-trip
- GIVEN a project with `push_enabled: true` and `consecutive_failures: 2` in `metadata.json`
- WHEN metadata is read, modified (unrelated field), and written back
- THEN `push_enabled` and `consecutive_failures` SHALL retain their values
