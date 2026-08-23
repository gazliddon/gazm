#!/usr/bin/env bash
#
# Local pre-publish sanity + version-bump flow. Runs BEFORE anything touches
# GitHub: proves gazm is healthy, bumps the patch version, keeps stargate's
# requires-gazm in lockstep, commits the three repos, and asks about pushing.
#
# Usage:
#   scripts/prepare-release.sh            # patch bump: 0.9.16 -> 0.9.17
#   scripts/prepare-release.sh minor      # minor bump: 0.9.16 -> 0.10.0
#   scripts/prepare-release.sh major      # major bump: 0.9.16 -> 1.0.0
#
# Exits non-zero on any failed check BEFORE bumping anything, so a broken
# tree never gets a version bump.
#
set -euo pipefail

BUMP="${1:-patch}"

GAZM_DIR="$(cd "$(dirname "$0")/.." && pwd)"
CRATES_DIR="$(cd "$GAZM_DIR/../crates" && pwd)"
STARGATE_DIR="$(cd "$GAZM_DIR/../stargate" && pwd)"

say() { printf '\033[1;36m==>\033[0m %s\n' "$*"; }
warn() { printf '\033[1;33mwarning:\033[0m %s\n' "$*"; }
die() { printf '\033[1;31merror:\033[0m %s\n' "$*" >&2; exit 1; }

confirm() { # $1 prompt; returns 0 if yes
  local ans
  read -r -p "$1 [y/N] " ans || true
  [[ "$ans" =~ ^[Yy]$ ]]
}

cd "$GAZM_DIR"

# ---------- 1. sanity: gazm tests ----------
say "Running gazm tests"
cargo test -p gazm
cargo fmt --all -- --check

# ---------- 1b. sanity: stargate assembles + checksums (if fixture present) ----------
if [ -d "$STARGATE_DIR/.git" ]; then
  say "Assembling Stargate fixture and verifying ROM checksums"
  # Assemble in a temp copy so we never touch the fixture's own roms/ dir
  # (gitignored output). The version check still runs: gazm reads
  # requires-gazm from the copied gazm.toml.
  TMP_SG="$(mktemp -d)"
  ( cd "$STARGATE_DIR" && git ls-files -z | tar --null -T - -cf - | ( cd "$TMP_SG" && tar -xf - ) )
  (
    cd "$TMP_SG"
    "$GAZM_DIR/target/debug/gazm" build 2>&1 | tee /tmp/gazm-prepare-build.log
    # gazm logs 'Cannot load binary ref' for the gitignored orig/roms on a
    # fresh checkout; that is expected. The checksum manifest is the gate.
    if grep -q "requires gazm" /tmp/gazm-prepare-build.log; then
      die "stargate requires a newer gazm than this binary"
    fi
    sha1sum -c roms.sha1
  )
  rm -f /tmp/gazm-prepare-build.log
  rm -rf "$TMP_SG"
else
  warn "stargate checkout not found at $STARGATE_DIR — skipping fixture check"
fi

# ---------- 2. decide whether to bump ----------
# Bump only when the gazm working tree has uncommitted changes. This makes
# prepare idempotent: running it twice in a row (after a prepare was already
# committed and pushed) does NOT march the version forward. Use --force to
# bump anyway (e.g. after committing work manually without running prepare).
cd "$GAZM_DIR"
DIRTY="$(git status --porcelain | wc -l | tr -d ' ')"
BUMP_NEEDED=1
FORCE_BUMP="${FORCE_BUMP:-0}"
if [ "$DIRTY" = "0" ] && [ "$FORCE_BUMP" = "0" ]; then
  warn "gazm working tree is clean — skipping version bump."
  warn "If the current version ($(grep -m1 '^version' "$GAZM_DIR/gazm/Cargo.toml" | sed -E 's/version = "([^"]+)"/\1/')) is ready, go straight to:"
  echo "  scripts/release.sh <version> --publish"
  echo "  (or re-run prepare with FORCE_BUMP=1 to bump despite a clean tree)"
  BUMP_NEEDED=0
fi

if [ "$BUMP_NEEDED" = "1" ]; then
  CURRENT="$(grep -m1 '^version' "$GAZM_DIR/gazm/Cargo.toml" | sed -E 's/version = "([^"]+)"/\1/')"
  IFS='.' read -r MAJOR MINOR PATCH <<< "$CURRENT"
  case "$BUMP" in
    major) NEW_MAJOR=$((MAJOR + 1)); NEW_MINOR=0; NEW_PATCH=0 ;;
    minor) NEW_MAJOR=$MAJOR; NEW_MINOR=$((MINOR + 1)); NEW_PATCH=0 ;;
    patch) NEW_MAJOR=$MAJOR; NEW_MINOR=$MINOR; NEW_PATCH=$((PATCH + 1)) ;;
    *) die "unknown bump type: $BUMP (use major|minor|patch)" ;;
  esac
  NEW_VERSION="$NEW_MAJOR.$NEW_MINOR.$NEW_PATCH"

  say "Bumping gazm $CURRENT -> $NEW_VERSION"
  sed -i '' "s/^version = \"$CURRENT\"/version = \"$NEW_VERSION\"/" "$GAZM_DIR/gazm/Cargo.toml"

  # keep stargate's requires-gazm in lockstep
  if [ -f "$STARGATE_DIR/gazm.toml" ]; then
    say "Updating stargate requires-gazm to $NEW_VERSION"
    sed -i '' "s/^requires-gazm = \".*\"/requires-gazm = \"$NEW_VERSION\"/" "$STARGATE_DIR/gazm.toml"
  fi
else
  # No gazm bump, but crates/stargate may still have independent changes.
  CURRENT="$(grep -m1 '^version' "$GAZM_DIR/gazm/Cargo.toml" | sed -E 's/version = "([^"]+)"/\1/')"
  NEW_VERSION="$CURRENT"
fi

# ---------- 4/5. commit each repo that has changes ----------
commit_repo() { # $1 dir, $2 message
  local dir="$1" msg="$2"
  ( cd "$dir" && git add -A && git diff --cached --quiet ) && return 0 # nothing staged
  ( cd "$dir" && git commit -q -m "$msg" )
  echo "  committed $(basename "$dir"): $msg"
}

say "Committing gazm"
if [ "$BUMP_NEEDED" = "1" ]; then
  commit_repo "$GAZM_DIR" "Bump version to $NEW_VERSION"
else
  commit_repo "$GAZM_DIR" "gazm: pre-release state for $NEW_VERSION"
fi

if [ -d "$STARGATE_DIR/.git" ]; then
  say "Committing stargate"
  if [ "$BUMP_NEEDED" = "1" ]; then
    commit_repo "$STARGATE_DIR" "Require gazm >= $NEW_VERSION"
  else
    commit_repo "$STARGATE_DIR" "stargate: pre-release state for gazm $NEW_VERSION"
  fi
fi

if [ -d "$CRATES_DIR/.git" ]; then
  say "Committing crates"
  commit_repo "$CRATES_DIR" "crates: pre-release state for gazm $NEW_VERSION"
fi

# ---------- 6. ask about pushing ----------
say "Done locally. Push?"
for repo in "$GAZM_DIR" "$CRATES_DIR" "$STARGATE_DIR"; do
  [ -d "$repo/.git" ] || continue
  name="$(basename "$repo")"
  branch="$(cd "$repo" && git branch --show-current)"
  unpushed="$(cd "$repo" && git log --oneline "@{u}..HEAD" 2>/dev/null | wc -l | tr -d ' ' || echo 0)"
  if [ "$unpushed" -gt 0 ] && confirm "  push $name ($unpushed commit(s)) to origin/$branch?"; then
    ( cd "$repo" && git push origin "$branch" )
    echo "  pushed $name"
  else
    echo "  skipped $name"
  fi
done

say "Next: scripts/release.sh $NEW_VERSION --publish (after CI goes green)"
