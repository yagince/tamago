---
name: release
description: >
  Semantic versioning release workflow for tamago using cargo-release.
  Analyzes git diff since last tag, proposes version bump (patch/minor/major),
  confirms with user via AskUserQuestion, then tags, pushes, waits for CI,
  and updates the Homebrew formula.
  Triggers: "release", "リリース", "バージョン上げて", "version bump", "tag"
---

# Release Workflow

## Step 1: Analyze Changes

```bash
git describe --tags --abbrev=0 2>/dev/null || echo "none"
git log $(git describe --tags --abbrev=0 2>/dev/null || git rev-list --max-parents=0 HEAD)..HEAD --oneline
```

## Step 2: Propose Version

Classify commits: `feat:` → minor, `fix:/refactor:/chore:` → patch, `BREAKING` → major.

Use **AskUserQuestion** to confirm version with diff summary.

## Step 3: Execute Release

If version differs from Cargo.toml:
```bash
cargo release <level> --no-publish --execute
```

If version matches Cargo.toml (e.g. first release):
```bash
git tag v<version>
git push origin main --tags
```

## Step 4: Wait for CI

Poll until the release workflow completes:
```bash
gh run list --workflow=release.yml --limit=1
```

Repeat every 15 seconds until status is `completed`. Report failure if it fails.

## Step 5: Update Homebrew Formula

After CI creates the GitHub Release with artifacts:

1. Download SHA256 checksums:
```bash
VERSION=<new_version>
ARM_MAC_SHA=$(gh release download v$VERSION --pattern "*aarch64-apple-darwin*.sha256" --output - | awk '{print $1}')
X86_MAC_SHA=$(gh release download v$VERSION --pattern "*x86_64-apple-darwin*.sha256" --output - | awk '{print $1}')
ARM_LINUX_SHA=$(gh release download v$VERSION --pattern "*aarch64-unknown-linux-musl*.sha256" --output - | awk '{print $1}')
X86_LINUX_SHA=$(gh release download v$VERSION --pattern "*x86_64-unknown-linux-musl*.sha256" --output - | awk '{print $1}')
```

2. Update `Formula/tamago.rb` in the homebrew-tamago repo at `../homebrew-tamago/`:
   - Update `version` to new version
   - Update all 4 `sha256` values

3. Commit and push:
```bash
cd ../homebrew-tamago
git add Formula/tamago.rb
git commit -m "update tamago to v$VERSION"
git push origin main
```

## Step 6: Report

Print:
- GitHub Release URL: `https://github.com/yagince/tamago/releases/tag/v<version>`
- Homebrew install command: `brew tap yagince/tamago && brew install tamago`

## Notes

- `publish = false` — crates.io publish is skipped
- Homebrew repo: `../homebrew-tamago/` (sibling directory)
- Always use `--no-publish` with cargo release
