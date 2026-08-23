# Releasing gazm

How to cut a new gazm release and publish binaries for Windows, macOS, and
Linux. The heavy lifting is automated by GitHub Actions; the only manual step
you keep control of is actually publishing the GitHub release.

## One command (recommended)

`scripts/release.sh` wraps the whole loop below — bump, commit, push, tag,
wait for the build, download artifacts — and defaults to **not publishing**:

```sh
scripts/release.sh 0.9.17                  # build only, artifacts in .release-artifacts/
scripts/release.sh 0.9.17 --publish        # also create the release + upload binaries
scripts/release.sh 0.9.17-preview --preview  # build via Actions tab trigger, no tag, no publish
```

It pre-flights everything: `gh` auth, clean gazm tree, and that `crates` and
`stargate` are pushed (CI fetches them from GitHub at run time). The sections
below describe what it does step by step.

## Prerequisites (one-time)

- `gh` CLI installed and authenticated (`gh auth login`).
- Both repos are public, so Actions minutes are free and no extra secrets are
  needed: `gazliddon/gazm`, `gazliddon/crates`, `gazliddon/stargate`.

## The release loop

```sh
# 1. Bump the version (semver): 0.9.16 -> 0.9.17
cargo set-version 0.9.17 --manifest-path gazm/Cargo.toml

# 2. Commit and push. CI (.github/workflows/ci.yml) runs the test suite and
#    the Stargate byte-identity check on master; wait for it to go green.
git add gazm/Cargo.toml
git commit -m "Bump version to 0.9.17"
git push origin master

# 3. Tag and push. This triggers .github/workflows/release.yml.
git tag v0.9.17
git push origin v0.9.17
```

## When you've also changed `crates` and/or `stargate`

The common case is a release that came out of working across all three repos.
Ordering matters: CI and the release build **check out crates and stargate
fresh from GitHub at run time** — your local changes to those repos don't
exist in CI until you push them. So push the shared pieces first:

```sh
# 1. Push the shared repos you touched (crates, and/or stargate fixture).
#    Only gazm is ever tagged; crates and stargate are never tagged.
git -C ~/development/crates add -A && git -C ~/development/crates commit -m "..." && git -C ~/development/crates push
git -C ~/development/stargate add -A && git -C ~/development/stargate commit -m "..." && git -C ~/development/stargate push

# 2. Then commit + bump gazm, as above. ci.yml now validates gazm against
#    the exact crates/stargate you just pushed; wait for it to go green.
# 3. Tag gazm and push; the release builds against crates@main as of that
#    moment (SHA recorded in each job summary).
```

If you want to preview the release **build** without committing a tag at all,
use the **manual trigger**: repo → Actions → Release workflow → "Run
workflow" → optionally set a preview version label (e.g. `0.9.17-preview`).
It builds all three targets into run artifacts exactly like a tag push, but
never creates a tag or release. Perfect for checking "does this build and
what do the binaries look like" before committing to the version number.

## What the release workflow does (and does not)

- Triggered by a `v*` tag push **or manually** from the Actions tab
  (`workflow_dispatch`, for previewing without a tag). Either way it builds
  **three targets** in parallel:
  `x86_64-unknown-linux-gnu` (Linux), `x86_64-pc-windows-msvc` (Windows),
  `aarch64-apple-darwin` (Apple Silicon).
- Checks out `gazliddon/crates` and `gazliddon/stargate` at their current
  `main`, and writes a **`build-manifest.txt`** artifact recording all three
  revisions (`gazm=<sha>`, `crates=<sha>`, `stargate=<sha>`), so every
  release states exactly which crates the binary was built against and which
  Stargate fixture the ROM checksums were verified against.
- Uses `taiki-e/upload-rust-binary-action` in **dry-run mode**: it builds and
  compresses the binaries but does **not** upload them anywhere.
- Uploads the `.tar.gz` / `.zip` / `.sha256` archives as **run artifacts**
  (downloadable from the Actions run page).

It deliberately does **not** create a GitHub release. Publishing stays manual:

## Publishing manually

1. Open the Actions run for the `v0.9.17` tag (from the repo → Actions →
   Release workflow).
2. Download the three `gazm-<target>` artifact sets (each contains
   `.tar.gz`/`.zip` + `.sha256`), plus `build-manifest.txt`.
3. Create the release and attach the archives:

```sh
# Create the release (notes: copy gazm/crates/stargate SHAs from build-manifest.txt)
gh release create v0.9.17 --title "v0.9.17" \
  --notes "Built from gazm@<sha> with crates@<sha> (ROMs verified against stargate@<sha>)"

# Upload each archive (adjust paths to your Downloads)
gh release upload v0.9.17 \
  gazm-x86_64-unknown-linux-gnu.tar.gz gazm-x86_64-unknown-linux-gnu.tar.gz.sha256 \
  gazm-x86_64-pc-windows-msvc.zip    gazm-x86_64-pc-windows-msvc.zip.sha256 \
  gazm-aarch64-apple-darwin.tar.gz   gazm-aarch64-apple-darwin.tar.gz.sha256
```

4. Verify the release page shows all three platforms + checksums.

## Rebuilding an old release exactly

Every job summary records the `crates@<sha>` used. To reproduce a release,
check out gazm at the tag and crates at that SHA locally:

```sh
git clone git@github.com:gazliddon/gazm.git
git -C gazm checkout v0.9.17
git clone git@github.com:gazliddon/crates.git ../crates   # alongside gazm/
git -C ../crates checkout <sha>
cd gazm && cargo build --release -p gazm
```

## Changing to full auto-publish (if you ever want it)

Edit `.github/workflows/release.yml`: set `dry_run: false` on the
upload-rust-binary step and add `permissions: contents: write` to the job.
Tag pushes will then build **and** attach binaries to an auto-created
release. (Consider switching the action to its companion
`taiki-e/create-gh-release-action` if you want a release body + changelog.)

## Notes

- gazm has no platform-specific dependencies, so the three targets cover all
  supported platforms; add targets to the matrix as needed.
- The Stargate check in `ci.yml` is the primary compatibility gate — a tag
  should only be cut after master is green.
- `crates` is intentionally not tagged/published; releases pin it by SHA
  instead (see `docs/AGENT_TRAIL.md` for why publishing was walked back).
