# Delta for project-settings-dialog

## MODIFIED Requirements

### Requirement: Identity Panel

The Identity panel MUST pre-fill name, email, and GitHub username from detected git identity when available, falling back to `cargarIdentidadGit` stored config, then to language-aware presets. It SHALL invoke `guardar_identidad_git` on save. Validation rules unchanged.
(Previously: pre-filled from `cargarIdentidadGit` only)

#### Scenario: Detected identity shown

- GIVEN detected git identity "Ada <ada@dev>" differs from stored config
- WHEN Identity panel opens
- THEN fields show detected name "Ada" and email "ada@dev"
- AND user MAY edit and save via `guardar_identidad_git`

#### Scenario: Stored config fallback

- GIVEN no detected git identity, stored config exists with "Bob <bob@dev>"
- WHEN Identity panel opens
- THEN fields show stored values "Bob" and "bob@dev" (unchanged behavior)

### Requirement: Remote Panel

The Remote panel MUST pre-fill the URL input from detected origin remote when available, falling back to stored remote config. On save, it SHALL invoke `configurar_remoto` then `guardar_config_remoto`. URL format validation rules unchanged.
(Previously: empty URL input on open — no pre-fill)

#### Scenario: Detected origin shown

- GIVEN detected origin `git@github.com:user/repo.git`
- WHEN Remote panel opens
- THEN URL input shows `git@github.com:user/repo.git`

#### Scenario: Empty when no origin detected

- GIVEN no detected origin and no stored remote config
- WHEN Remote panel opens
- THEN URL input is empty (unchanged behavior)
