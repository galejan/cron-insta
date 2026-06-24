# Delta for git-remote-sync

## MODIFIED Requirements

### Requirement: Remote Configuration and Initial Push

On valid SSH URL: `git remote add origin <url>` then `git push -u origin main`. Success writes `push_enabled: true` and `consecutive_failures: 0` to project's `.config/metadata.json`. Remote URL SHALL be read dynamically via `git remote get-url origin`, not stored in config.
(Previously: state stored in global `git-config.json`; URL stored in global config)

#### Scenario: Remote added and pushed successfully
- GIVEN new project with valid SSH URL and accessible remote
- WHEN user confirms remote configuration
- THEN `origin` remote configured, push succeeds, and `push_enabled: true`, `consecutive_failures: 0` written to project's `metadata.json`

#### Scenario: Remote not accessible on first push
- GIVEN valid SSH URL but remote unreachable
- WHEN initial push attempted
- THEN commit remains local, user receives non-blocking warning, `push_enabled` stays `true` in project's `metadata.json`

#### Scenario: Per-project push state isolation
- GIVEN project A with `push_enabled: true`, project B with no remote
- WHEN user configures remote on project B
- THEN project A's state unchanged; project B's state independent

### Requirement: Checkpoint Auto-Push

When `push_enabled: true` in project's `metadata.json`, `crear_checkpoint` SHALL push after each successful commit. Remote URL read from `git remote get-url origin`. Push failures increment `consecutive_failures` in `metadata.json`. After 3 consecutive failures, `push_enabled` set to `false` and user notified. Successful push resets counter to 0.
(Previously: state stored in global `git-config.json`)

#### Scenario: Push succeeds silently
- GIVEN `push_enabled: true` and successful checkpoint commit
- WHEN auto-push executes → THEN no notification appears

#### Scenario: First push failure (strike 1)
- GIVEN `push_enabled: true` and remote inaccessible
- WHEN auto-push fails → THEN warning toast appears; `consecutive_failures` increments to 1 in project's `metadata.json`

#### Scenario: Third consecutive failure → disable
- GIVEN `push_enabled: true` and 2 prior failures in `metadata.json`
- WHEN push fails third time → THEN `push_enabled` set to `false` in project's `metadata.json`; user notified

#### Scenario: Success resets counter
- GIVEN 1–2 prior failures in `metadata.json`
- WHEN next push succeeds → THEN `consecutive_failures` resets to 0 in project's `metadata.json`

#### Scenario: Failure isolation between projects
- GIVEN project A has 2 failures, project B has 0
- WHEN push fails for project A → THEN project A's counter increments, project B's unchanged

### Requirement: Toolbar Warning and Re-enable Flow

The system SHALL display ⚠️ in the toolbar only when the CURRENT loaded project has a configured remote but `push_enabled: false` in its `metadata.json`. Users who never configured a remote see no warning. Re-enabling writes to project's `metadata.json` and resets `consecutive_failures` to 0.
(Previously: warning/re-enable read/wrote global `git-config.json`)

#### Scenario: Warning shown for disabled remote
- GIVEN project with configured remote and `push_enabled: false`
- WHEN toolbar renders → THEN ⚠️ icon visible

#### Scenario: No warning without remote config
- GIVEN project where no remote was ever configured
- WHEN toolbar renders → THEN no ⚠️ icon appears

#### Scenario: User re-enables push
- GIVEN `push_enabled: false` after 3 consecutive failures
- WHEN user clicks ⚠️ and chooses "Retry Sync"
- THEN `push_enabled` set to `true` in project's `metadata.json`; `consecutive_failures` resets to 0

#### Scenario: Warning vanishes when switching to healthy project
- GIVEN project A loaded with push disabled (⚠️ shown), project B with push enabled
- WHEN user opens project B → THEN ⚠️ disappears
