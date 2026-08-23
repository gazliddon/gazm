#!/usr/bin/env bash
#
# One-command gazm release loop.
#
# The version is READ from gazm/Cargo.toml — it is set by
# scripts/prepare-release.sh, which owns all version bumps (and keeps
# stargate's requires-gazm in lockstep). This script never edits the
# version; it tags the current one, builds, downloads, and optionally
# publishes.
#
# Usage:
#   scripts/release.sh                    build (dry-run), download artifacts — NO publish, NO tag
#   scripts/release.sh --publish          tag + build + create GitHub release + upload archives
#   scripts/release.sh --preview          build only via workflow_dispatch — no tag, no publish
#
# Pre-flight: verifies crates/stargate are pushed (CI fetches them from GitHub),
# the gazm tree is clean, and gh is authenticated.
#
set -euo pipefail

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

# Version comes from Cargo.toml (prepare-release.sh owns bumping).
VERSION="$(grep -m1 '^version' "$GAZM_DIR/gazm/Cargo.toml" | sed -E 's/version = "([^"]+)"/\1/')"
[ -n "$VERSION" ] || die "could not read version from gazm/Cargo.toml"

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

# ---------- tag (only for a real release; preview never tags) ----------
if [ "$PREVIEW" = "1" ]; then
  say "Preview build (workflow_dispatch) — no tag, no publish (version $VERSION)"
elif git rev-parse -q --verify "refs/tags/v$VERSION" >/dev/null; then
  die "tag v$VERSION already exists — this version was already released. Bump via prepare-release.sh."
else
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

# Build notes: prefer the full RELEASE_NOTES.md written by the workflow
# (includes the source revisions AND the end-user verification steps),
# falling back to a one-line summary with the local SHAs.
NOTES_FILE="$(find .release-artifacts -name RELEASE_NOTES.md | head -1)"
MANIFEST="$(find .release-artifacts -name build-manifest.txt | head -1)"
if [ -n "$MANIFEST" ]; then
  GAZM_SHA="$(sed -n 's/^gazm=//p' "$MANIFEST")"
  CRATES_SHA="$(sed -n 's/^crates=//p' "$MANIFEST")"
  STARGATE_SHA="$(sed -n 's/^stargate=//p' "$MANIFEST")"
else
  GAZM_SHA="$(git rev-parse HEAD)"
  STARGATE_SHA="$(git -C "$STARGATE_DIR" rev-parse HEAD 2>/dev/null || echo unknown)"
fi

# ---------- publish (only with --publish) ----------
if [ "$PUBLISH" = "1" ]; then
  if [ "$PREVIEW" = "1" ]; then
    die "--publish with --preview: publish a real tag instead (drop --preview)"
  fi
  say "Creating release v$VERSION"
  if [ -n "$NOTES_FILE" ]; then
    gh release create "v$VERSION" --title "v$VERSION" --notes-file "$NOTES_FILE"
  else
    NOTES="Built from gazm@${GAZM_SHA} with crates@${CRATES_SHA} (ROMs verified against stargate@${STARGATE_SHA})"
    echo "$NOTES"
    gh release create "v$VERSION" --title "v$VERSION" --notes "$NOTES"
  fi
  find .release-artifacts -type f -print0 | while IFS= read -r -d '' f; do
    gh release upload "v$VERSION" "$f"
  done
  say "Done: https://github.com/gazliddon/gazm/releases/tag/v$VERSION"
else
  say "Build complete — artifacts in .release-artifacts/"
  say "To publish manually:"
  if [ -n "$NOTES_FILE" ]; then
    echo "  gh release create v$VERSION --title v$VERSION --notes-file '$NOTES_FILE'"
  else
    echo "  gh release create v$VERSION --notes \"Built from gazm@${GAZM_SHA} with crates@${CRATES_SHA} (ROMs verified against stargate@${STARGATE_SHA})\""
  fi
  echo "  gh release upload v$VERSION .release-artifacts/*/*"
fi
