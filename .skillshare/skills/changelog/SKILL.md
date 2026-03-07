---
name: veto-changelog
description: >-
  Generate CHANGELOG.md entry from recent commits in conventional format. Use
  this skill whenever the user asks to: write release notes, generate a
  changelog, prepare a version release, document what changed between tags, or
  create a new CHANGELOG entry. Do NOT manually edit CHANGELOG.md without this
  skill — it ensures proper formatting, user-perspective writing, and version
  consistency with Cargo.toml.
argument-hint: "[tag-version]"
targets: [claude]
---

Generate a CHANGELOG.md entry for a release. $ARGUMENTS specifies the tag version (e.g., `v0.1.11`) or omit to auto-detect via `git describe --tags --abbrev=0`.

**Scope**: This skill updates `CHANGELOG.md` only. It does NOT write Rust code or modify Cargo.toml.

## Workflow

### Step 1: Determine Version Range

```bash
# Auto-detect latest tag
LATEST_TAG=$(git describe --tags --abbrev=0)
PREV_TAG=$(git describe --tags --abbrev=0 "${LATEST_TAG}^")
CARGO_VERSION=$(grep '^version' Cargo.toml | head -1 | cut -d'"' -f2)

echo "Generating changelog: $PREV_TAG → $LATEST_TAG"
echo "Cargo.toml version: $CARGO_VERSION"
```

Verify version consistency:
- Tag version (without `v` prefix) should match `Cargo.toml` version
- Warn if mismatch (the pre-push hook also checks this)

### Step 2: Collect Commits

```bash
git log "${PREV_TAG}..${LATEST_TAG}" --oneline --no-merges
```

### Step 3: Categorize Changes

Group by conventional commit type:

| Prefix | Category |
|--------|----------|
| `feat` | New Features |
| `fix` | Bug Fixes |
| `refactor` | Refactoring |
| `style` | Code Style |
| `perf` | Performance |
| `test` | Tests |
| `chore` | Maintenance |

### Step 4: Read Existing Style

Read the most recent 2-3 entries in `CHANGELOG.md` to match tone and structure. Always match the latest entries.

### Step 5: Write User-Facing Entry

Write from the **user's perspective**:

**Include**:
- New rules or risk categories with examples
- New CLI commands or flags with usage
- New integrations (setup commands)
- Bug fixes affecting user-visible behavior
- Auth method changes

**Exclude**:
- Internal test refactoring
- Code formatting changes
- CI/CD tweaks
- Internal struct renames

### Step 6: Update CHANGELOG.md

Insert new entry at the top, after the header:

```markdown
## [X.Y.Z] - YYYY-MM-DD

### New Features

- **Feature name** — description with `inline code` for commands
  ```bash
  veto command --flag    # usage example
  ```

### Bug Fixes

- Fixed specific user-visible behavior

### Breaking Changes

- Renamed `old-flag` to `new-flag`
```

Style rules:
- Version uses `[X.Y.Z]` without `v` prefix in heading
- Feature bullets use `**bold name** — em-dash description`
- Code blocks use `bash` language tag
- Only include sections that have content

## Rules

- **User perspective** — write for users, not developers
- **No fabricated links** — never invent URLs
- **Verify features exist** — grep source before claiming a feature was added
- **No internal noise** — exclude test-only, CI-only, or style-only changes
- **Version consistency** — verify Cargo.toml matches tag version
