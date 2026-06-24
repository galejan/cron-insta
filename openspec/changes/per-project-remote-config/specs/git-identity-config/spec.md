# Delta for git-identity-config

## MODIFIED Requirements

### Requirement: Global Identity Storage

The system MUST store Git identity in the platform app config directory at `cron-insta/git-config.json`. The config file SHALL contain ONLY identity fields: `schema_version`, `name`, `email`, and `github_user`. It MUST NOT contain a `remote` section.

On first load after this change, any existing `remote` key SHALL be stripped from the config file (migration). This migration is idempotent — subsequent loads SHALL NOT error if `remote` is already absent.
(Previously: config included a `remote` section with `url`, `push_enabled`, and `consecutive_failures`)

#### Scenario: First project — no config exists
- GIVEN no `git-config.json` exists in the app config dir
- WHEN a new project is created
- THEN the identity dialog SHALL pre-fill with a language-aware preset:
  - Spanish UI → name: "Miguel de Cervantes", email: "cervantes@literatura.es"
  - English UI → name: "William Shakespeare", email: "shakespeare@literature.en"
- AND the user MAY accept or customize these values

#### Scenario: Subsequent project — config exists
- GIVEN `git-config.json` exists with `{"name":"Ada Lovelace","email":"ada@code.dev"}`
- WHEN a new project is created
- THEN the identity dialog SHALL pre-fill with the stored name and email

#### Scenario: User customizes identity
- GIVEN the identity dialog is shown with any pre-filled values
- WHEN the user modifies name and/or email and confirms
- THEN the new values MUST be saved to `git-config.json`
- AND `git config user.name` and `git config user.email` SHALL be set in the project repo

#### Scenario: User accepts defaults unchanged
- GIVEN the identity dialog is shown with pre-filled values (preset or prior config)
- WHEN the user confirms without changes
- THEN the values MUST be saved to `git-config.json`
- AND the repo identity SHALL be configured accordingly

#### Scenario: Config file corrupted
- GIVEN `git-config.json` exists but contains invalid JSON
- WHEN the system attempts to load identity
- THEN the system MUST fall back to language-aware presets
- AND no crash or unhandled error SHALL occur

#### Scenario: Migration strips legacy remote section
- GIVEN `git-config.json` contains `{"name":"...","email":"...","remote":{"url":"...","push_enabled":true}}`
- WHEN the config is loaded by `cargarConfigRemoto`
- THEN the `remote` key SHALL be removed from the in-memory config
- AND identity fields SHALL be preserved
- AND the cleaned config SHALL be written back to disk

#### Scenario: Migration is idempotent
- GIVEN `git-config.json` has already been migrated (no `remote` key)
- WHEN the config is loaded again
- THEN no error SHALL occur
- AND identity fields SHALL remain intact

## REMOVED Requirements

None.
