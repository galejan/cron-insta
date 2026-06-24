# Delta for project-settings-dialog

## MODIFIED Requirements

### Requirement: Remote Panel

The Remote panel MUST display a URL input and a Save button. It SHALL pre-fill the URL from `git remote get-url origin` (dynamic read from the project's Git config) on panel open.

On save, it SHALL invoke `configurar_remoto(projectPath, url)` followed by `guardar_config_remoto(projectPath, url, true)`, where `projectPath` identifies the current project. It SHALL validate URL format client-side before invoking backend commands.
(Previously: used `guardar_config_remoto(url, true)` without `projectPath`; URL loaded from global config, not git)

#### Scenario: Set valid remote URL
- GIVEN the Remote panel is open with a project that has no remote set
- WHEN the user enters `git@github.com:user/repo.git` and clicks Save
- THEN `configurar_remoto(projectPath, url)` is invoked with the current project path
- AND `guardar_config_remoto(projectPath, url, true)` persists push state per-project

#### Scenario: Reject invalid URL format
- GIVEN the Remote panel is open
- WHEN the user enters `not-a-valid-url` and clicks Save
- THEN a validation error is shown before any backend call
- AND neither `configurar_remoto` nor `guardar_config_remoto` is invoked

#### Scenario: Remote URL pre-filled from git config
- GIVEN a project with origin remote set to `git@github.com:user/repo.git`
- WHEN the Remote panel opens
- THEN the URL input SHALL display `git@github.com:user/repo.git` (read from git)
