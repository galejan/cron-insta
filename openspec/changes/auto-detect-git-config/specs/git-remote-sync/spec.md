# Delta for git-remote-sync

## ADDED Requirements

### Requirement: Auto-Enable Push on Detected Remote

When `detectar_config_git` finds an SSH remote URL, the system SHALL set `push_enabled: true` in project metadata. Auto-enable MUST be skipped when `consecutive_failures > 0` (3-strike rule). Non-SSH URLs SHALL NOT auto-enable push — the SSH-only rule applies to auto-detection.

#### Scenario: SSH origin detected → push enabled

- GIVEN detected origin `git@github.com:user/repo.git` and `consecutive_failures = 0`
- WHEN project opens and detection completes
- THEN `push_enabled` set to `true`

#### Scenario: HTTPS origin → no auto-enable

- GIVEN detected origin `https://github.com/user/repo.git`
- WHEN detection completes
- THEN `push_enabled` remains unchanged

#### Scenario: 3-strike guard blocks auto-enable

- GIVEN detected SSH origin and `consecutive_failures > 0`
- WHEN detection completes
- THEN `push_enabled` remains `false`
