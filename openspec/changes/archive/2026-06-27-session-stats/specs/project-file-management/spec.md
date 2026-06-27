# Delta for project-file-management

## MODIFIED Requirements

### Requirement: Project Folder Creation

The system MUST create a complete project directory structure when `crear_proyecto` is invoked.

Given a root path and project name, the system SHALL:
- Create subdirectories: `.config/`, `capitulos/`, `personajes/`, `notas/`, `lugares/`
- Write `.config/metadata.json` with seed schema: `{ project_name, last_modified (ISO 8601), chapters_order: [], characters_index: [], places_index: [] }`
- Write `.config/timeline.json` as empty array `[]`
- Write `.config/stats.json` with seed schema: `{ total_time_seconds: 0, total_words: 0, chapters: {}, sessions: [] }`
- Write `lugares/index.json` as empty array `[]`
- Return success with the project path on completion
- Return `Err(String)` for any I/O failure (permission denied, disk full, invalid path)
(Previously: Did not seed `.config/stats.json`)

#### Scenario: Creates project with stats seed

- GIVEN a writable directory `/tmp/test-project`
- WHEN `crear_proyecto("/tmp/test-project", "Mi Novela")` is called
- THEN all five subdirectories exist including `lugares/`
- AND `metadata.json` contains `"project_name": "Mi Novela"` with a valid ISO 8601 `last_modified`
- AND `stats.json` contains `{"total_time_seconds": 0, "total_words": 0, "chapters": {}, "sessions": []}`
- AND `lugares/index.json` contains `[]`
- AND `timeline.json` contains `[]`

#### Scenario: Rejects inaccessible path

- GIVEN a path `/root/blocked` where the process lacks write permission
- WHEN `crear_proyecto("/root/blocked", "Test")` is called
- THEN the function returns `Err(String)` describing the permission error
- AND no partial directory structure is left behind

#### Scenario: Handles path with trailing separator

- GIVEN path `/tmp/test-project/` with trailing `/`
- WHEN `crear_proyecto("/tmp/test-project/", "Test")` is called
- THEN the function normalises the path and creates the project correctly
