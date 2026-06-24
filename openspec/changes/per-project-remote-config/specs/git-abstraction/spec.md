# Delta for git-abstraction

## MODIFIED Requirements

### Requirement: Checkpoint Creation

The system SHALL create Git commits as versioned snapshots (Nivel 2 — deferred, not per-keystroke).

`crear_checkpoint` runs `git add .` followed by `git commit` with a descriptive message. When `push_enabled: true` in the project's `metadata.json` and `git remote get-url origin` returns a URL, it SHALL attempt `git push origin main` after successful commit. The `SSH_AUTH_SOCK` environment variable SHALL be passed to the git subprocess on Linux.

Push errors MUST be surfaced (returned as `Err`, not silently dropped). The caller SHALL update `consecutive_failures` in the project's `metadata.json` on failure. Triggered by a frontend inactivity timer (≥30 min idle, ≥100 words since last checkpoint).
(Previously: push_enabled read from global config; push errors silently dropped via `let _`; SSH_AUTH_SOCK not passed)

#### Scenario: Creates a checkpoint commit
- GIVEN a project with `.git/` initialized and modified chapter files
- WHEN `crear_checkpoint("/tmp/proj")` is called
- THEN all changes are staged and committed
- AND the commit message follows format: "Progreso automático: [date] - [word count]"
- AND the function returns `Ok` with the commit hash

#### Scenario: No changes to commit
- GIVEN a clean Git repo with no modified files
- WHEN `crear_checkpoint` is called
- THEN the function returns `Ok` with a message indicating no changes were committed
- AND no empty commit is created

#### Scenario: Checkpoint when Git unavailable
- GIVEN a project where Git is not available
- WHEN `crear_checkpoint` is called
- THEN the function returns `Err` with "Git no está disponible"
- AND disk files remain unaffected

### Requirement: Checkpoint with Auto-Push

The system SHALL attempt `git push origin main` after a successful checkpoint commit when `push_enabled: true` in the project's `metadata.json` and a remote is configured (detected via `git remote get-url origin`).

Push outcome SHALL be surfaced to the caller as `Result`. On push failure, the caller MUST increment `consecutive_failures` in the project's `metadata.json`. After 3 consecutive failures, `push_enabled` SHALL be set to `false` in `metadata.json`. A successful push SHALL reset `consecutive_failures` to 0. The `SSH_AUTH_SOCK` environment variable SHALL be passed to the push subprocess on Linux.
(Previously: push_enabled from global config; push errors dropped silently; no SSH_AUTH_SOCK passing)

#### Scenario: Checkpoint with push_enabled=true and accessible remote
- GIVEN `push_enabled: true` in project metadata and a reachable remote
- WHEN `crear_checkpoint` commits successfully
- THEN `git push origin main` SHALL execute with `SSH_AUTH_SOCK` passed
- AND the function SHALL return `Ok` with the push result

#### Scenario: Checkpoint with push_enabled=true but push fails
- GIVEN `push_enabled: true` and an unreachable remote
- WHEN `crear_checkpoint` commits successfully then push fails
- THEN the local commit SHALL remain intact
- AND the function SHALL return `Err` with the push failure reason
- AND the caller SHALL increment `consecutive_failures` in project metadata

#### Scenario: Checkpoint with push_enabled=false
- GIVEN `push_enabled: false` in project metadata
- WHEN `crear_checkpoint` commits successfully
- THEN no push SHALL be attempted

#### Scenario: Checkpoint when remote was never configured
- GIVEN no remote URL returned by `git remote get-url origin`
- WHEN `crear_checkpoint` commits successfully
- THEN no push SHALL be attempted
