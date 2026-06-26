# Releasing AgentPack

## Overview

Releases are automated via GitHub Actions. Pushing a version tag triggers the full pipeline: lint → test → security → build → release.

## Process

### 1. Prepare the release

```bash
# Ensure you're on master with clean state
git checkout master
git pull origin master

# Bump version in Cargo.toml
# e.g. version = "0.2.0"
vim Cargo.toml

# Update Cargo.lock
cargo check

# Verify everything passes
cargo fmt --check
cargo clippy -- -D warnings -D clippy::unwrap_used -D clippy::panic
cargo test -- --test-threads=1
```

### 2. Commit the version bump

```bash
git add Cargo.toml Cargo.lock
git commit -m "chore: bump version to 0.2.0"
git push origin master
```

### 3. Tag and push

```bash
git tag -a v0.2.0 -m "v0.2.0 - Short description

- Feature A
- Feature B
- Fix C"

git push origin v0.2.0
```

### 4. What happens automatically

The `v*` tag triggers the CI pipeline:

```
Tag pushed
  │
  ├─► lint (fmt + clippy strict)
  ├─► test (cargo test)
  ├─► security-audit (cargo-audit / rustsec)
  ├─► secrets-scan (TruffleHog)
  ├─► sast (CodeQL)
  ├─► license-check (cargo-deny)
  │
  ▼ all pass
  │
  ├─► build (4 binaries)
  │   ├─ agentpack-linux-amd64
  │   ├─ agentpack-linux-arm64
  │   ├─ agentpack-darwin-amd64
  │   └─ agentpack-darwin-arm64
  │
  ▼ all pass
  │
  └─► release
      ├─ Creates GitHub Release with tag name
      ├─ Attaches all 4 binaries
      └─ Auto-generates release notes from commits
```

### 5. Post-release

- Verify the release appears at https://github.com/ricardosalcedo/agentpack/releases
- Update the Homebrew formula SHA256 hashes:

```bash
# Download each binary and compute hash
curl -fsSL https://github.com/ricardosalcedo/agentpack/releases/download/v0.2.0/agentpack-darwin-arm64 | shasum -a 256
```

- Update `Formula/agentpack.rb` with the new version and hashes
- Announce (see SOCIALIZATION.md)

## Versioning

We follow [Semantic Versioning](https://semver.org/):

| Change type | Version bump | Example |
|-------------|-------------|---------|
| Breaking manifest/CLI change | Major (1.0.0) | Rename `dependencies` field |
| New command or feature | Minor (0.2.0) | Add `agentpack run` |
| Bug fix | Patch (0.1.1) | Fix version parsing |

Pre-1.0: breaking changes bump minor (0.1.0 → 0.2.0).

## Hotfix process

```bash
# Branch from the tag
git checkout -b hotfix/0.1.1 v0.1.0

# Fix, commit
git commit -m "fix: description"

# Tag and push
git tag -a v0.1.1 -m "v0.1.1 - hotfix"
git push origin v0.1.1

# Merge back to master
git checkout master
git merge hotfix/0.1.1
git push origin master
```

## Security gate

Releases **will not be created** unless all of these pass:

- `cargo-audit` — no known vulnerabilities in dependencies
- TruffleHog — no secrets in commit history
- CodeQL — no SAST findings
- `cargo-deny` — no disallowed licenses (GPL, etc.)
- clippy strict — no `unwrap()`, `panic!()`, or warnings
- All tests pass

If any check fails, fix the issue before re-tagging.

## Rolling back a release

```bash
# Delete the tag locally and remotely
git tag -d v0.2.0
git push origin :refs/tags/v0.2.0

# Delete the GitHub release via CLI
gh release delete v0.2.0 --yes
```
