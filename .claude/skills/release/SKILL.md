---
name: release
description: >
  Semantic versioning release workflow for tamago using cargo-release.
  Analyzes git diff since last tag, proposes version bump (patch/minor/major),
  confirms with user via AskUserQuestion, then runs cargo release to tag and push.
  GitHub Actions release.yml handles artifact builds automatically on tag push.
  Triggers: "release", "リリース", "バージョン上げて", "version bump", "tag"
---

# Release Workflow

## Step 1: Analyze Changes

Determine the last release tag and diff:

```bash
git describe --tags --abbrev=0 2>/dev/null || echo "none"
git log $(git describe --tags --abbrev=0 2>/dev/null || git rev-list --max-parents=0 HEAD)..HEAD --oneline
```

## Step 2: Propose Version

Classify changes by commit messages:
- **major**: breaking changes (`BREAKING`, incompatible API change)
- **minor**: new features (`feat:`)
- **patch**: bug fixes, refactors, chores (`fix:`, `refactor:`, `chore:`)

Read the current version from Cargo.toml. Calculate the proposed next version.

Use **AskUserQuestion** to confirm:
- Show the diff summary (commit list)
- Propose the version bump level and new version number
- Let the user choose: proposed version, or override

## Step 3: Execute Release

```bash
cargo release <level> --no-publish --execute
```

Where `<level>` is `patch`, `minor`, or `major`.

This will:
1. Bump version in Cargo.toml
2. Commit the version change
3. Create git tag `v<new_version>`
4. Push commit and tag to remote

## Step 4: Verify

After push, the tag triggers `.github/workflows/release.yml` which:
- Builds 4 platform binaries (macOS arm64/x86_64, Linux musl x86_64/aarch64)
- Creates GitHub Release with artifacts and checksums

Check the workflow status:
```bash
gh run list --workflow=release.yml --limit=1
```

Report the release URL to the user.

## Notes

- `publish = false` in Cargo.toml, so crates.io publish is skipped
- First release will have no previous tag; use all commits since init
- Always use `--no-publish` flag with cargo release
