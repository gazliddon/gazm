#!/usr/bin/env bash
#
# One-command gazm release loop.
#
# Usage:
#   scripts/release.sh 0.9.17                  bump, commit, push, tag, build (dry-run), download artifacts — NO publish
#   scripts/release.sh 0.9.17 --publish        same, then create the GitHub release and upload the archives
#   scripts/release.sh 0.9.17-preview --preview  build only via workflow_dispatch — no tag, no publish
#
# Pre-flight: verifies crates/stargate are pushed (CI fetches them from GitHub),
# the gazm tree is clean, and gh is authenticated.
#
set -euo pipefail

VERSION="${1:-}"
[ -n "$VERSION" ] || { echo "usage: $0 <version> [--publish] [--preview]" >&2; exit 1; }

PUBLISH=0
PREVIEW=0
for arg in "$@"; do
  case "$arg" in
    --publish) PUBLISH=1 ;;
    --preview) PREVIEW=1 ;;
  esac
done

GAZM_DIR="$(cd "$(dirname "$0")/.." && pwd)"
CRATES_DIR="$(cd "$GAZM_DIR/../crates" && pwd)"
STARGATE_DIR="$(cd "$GAZM_DIR/../stargate" && pwd)"

say() { printf '\033[1;36m==>\033[0m %s\n' "$*"; }
die() { printf '\033[1;31merror:\033[0m %s\n' "$*" >&2; exit 1; }

cd "$GAZM_DIR"

# ---------- pre-flight ----------
command -v gh >/dev/null || die "gh CLI not installed"
gh auth status >/dev/null 2>&1 || die "gh not authenticated"

git diff --quiet -- gazm/Cargo.toml || die "gazm/Cargo.toml has uncommitted changes; commit or stash first"
[ "$(git status --porcelain | wc -l | tr -d ' ')" = "0" ] || {
  echo "warning: gazm working tree is not clean:" >&2
  git status --short >&2
  read -r -p "Continue anyway? [y/N] " ans || true
  [[ "$ans" =~ ^[Yy]$ ]] || die "aborted"
}

# CI/release fetch crates and stargate from GitHub; make sure local == remote.
for repo in "$CRATES_DIR" "$STARGATE_DIR"; do
  if [ -d "$repo/.git" ]; then
    ( cd "$repo" && git fetch -q origin 2>/dev/null || true )
    local_head="$(cd "$repo" && git rev-parse HEAD 2>/dev/null || echo none)"
    remote_head="$(cd "$repo" && git rev-parse @{u} 2>/dev/null || echo none)"
    if [ "$local_head" != "$remote_head" ]; then
      echo "warning: $(basename "$repo") is not pushed (local $local_head vs remote $remote_head)." >&2
      echo "         CI checks out crates/stargate from GitHub — un-pushed changes won't be in the build." >&2
      read -r -p "Push $(basename "$repo") now? [y/N] " ans || true
      [[ "$ans" =~ ^[Yy]$ ]] || die "push $(basename "$repo") first"
      ( cd "$repo" && git push -q origin HEAD )
    fi
  fi
done

# ---------- bump ----------
if [ "$PREVIEW" = "1" ]; then
  say "Preview build (workflow_dispatch) — no version bump, no tag"
else
  say "Bumping gazm version to $VERSION"
  perl -pi -e "s/^version = \".*\"/version = \"$VERSION\"/" gazm/Cargo.toml
  git add gazm/Cargo.toml
  git commit -q -m "Bump version to $VERSION"
  say "Pushing master"
  git push -q origin master
  say "Tagging v$VERSION"
  git tag "v$VERSION"
  git push -q origin "v$VERSION"
fi

# ---------- build (dry-run) ----------
CRATES_SHA="$(git -C "$CRATES_DIR" rev-parse HEAD 2>/dev/null || echo unknown)"
if [ "$PREVIEW" = "1" ]; then
  say "Triggering manual release build (version label: $VERSION)"
  gh workflow run release.yml -f "version=$VERSION"
  sleep 5
fi
RUN_ID="$(gh run list --workflow=release.yml --limit 1 --json databaseId -q '.[0].databaseId')"
[ -n "$RUN_ID" ] && [ "$RUN_ID" != "null" ] || die "could not find the release workflow run"
say "Waiting for run $RUN_ID (this takes a few minutes)"
gh run watch "$RUN_ID" --exit-status >/dev/null 2>&1 || {
  gh run view "$RUN_ID" >&2
  die "release build failed — see the run above"
}

say "Downloading artifacts"
rm -rf .release-artifacts && mkdir -p .release-artifacts
gh run download "$RUN_ID" --dir .release-artifacts
find .release-artifacts -type f | sort | sed 's/^/  /'

# ---------- publish (only with --publish) ----------
if [ "$PUBLISH" = "1" ]; then
  if [ "$PREVIEW" = "1" ]; then
    die "--publish with --preview: publish a real tag instead (drop --preview)"
  fi
  say "Creating release v$VERSION (crates@${CRATES_SHA:0:12})"
  gh release create "v$VERSION" --title "v$VERSION" --notes "crates@$CRATES_SHA"
  find .release-artifacts -type f -print0 | while IFS= read -r -d '' f; do
    gh release upload "v$VERSION" "$f"
  done
  say "Done: https://github.com/gazliddon/gazm/releases/tag/v$VERSION"
else
  say "Build complete — artifacts in .release-artifacts/"
  say "To publish manually:"
  echo "  gh release create v$VERSION --notes 'crates@$CRATES_SHA'"
  echo "  gh release upload v$VERSION .release-artifacts/*/*"
fi
