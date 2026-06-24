# Delta for git-integration

## ADDED Requirements

### Requirement: detectar_config_git Command

The system MUST provide `detectar_config_git(path)` as a Tauri command returning `{name, email, remote_url}` from `.git/config`. Values SHALL be `null` when unavailable. The command MUST NOT error — it is best-effort only.

#### Scenario: Full config detected

- GIVEN `.git/config` has user.name, user.email, and `origin` remote
- WHEN `detectar_config_git` runs
- THEN returns `{name, email, remote_url}` with all values populated

#### Scenario: Missing remote

- GIVEN `.git/config` has user.name/email but no origin remote
- WHEN `detectar_config_git` runs
- THEN returns `{name, email, remote_url: null}`

#### Scenario: No .git directory

- GIVEN project path without `.git/`
- WHEN `detectar_config_git` runs
- THEN returns `{name: null, email: null, remote_url: null}`

### Requirement: Auto-Detection on Project Open

The system SHALL invoke `detectar_config_git` as a non-blocking background call on `abrirProyecto` and auto-reopen. Detected identity differing from stored global config SHALL update via `guardar_identidad_git`. Detected SSH remote with 0 consecutive failures SHALL set `push_enabled: true`. A toast SHALL announce: origin repo and identity when detected.

#### Scenario: Identity differs → update + toast

- GIVEN stored identity "Bob" and detected "Ada <ada@dev>"
- WHEN project opens and detection completes
- THEN `guardar_identidad_git("Ada", "ada@dev", ...)` is called
- AND toast shows "Git detectado: origin → user/repo, identidad → Ada <ada@dev>"

#### Scenario: Same identity → only origin toast

- GIVEN stored identity matches detected, origin remote detected
- WHEN detection completes
- THEN no `guardar_identidad_git` call
- AND toast shows origin repo

#### Scenario: No .git → silent no-op

- GIVEN project without `.git/`
- WHEN project opens
- THEN no toast, no config changes, no side effects
