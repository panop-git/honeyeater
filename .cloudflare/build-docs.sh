#!/usr/bin/env bash
# Build script used by Cloudflare Workers Builds to build the honeyeater
# narrative documentation (mdBook).
#
# Cloudflare's default build image does not include mdBook, so we fetch a
# pinned release into the build container before invoking it. The build
# output (./book/) is then published as static assets by Wrangler per the
# top-level wrangler.jsonc.
#
# Cloudflare dashboard settings to use:
#
#   Build command:        ./.cloudflare/build-docs.sh
#   Deploy command:       npx wrangler deploy   (default, no change needed)
#   Root directory:       /
#   Environment vars:     (none required)
#
# Pin the mdBook version here. Update when intentional; the CI workflow at
# .github/workflows/ci.yml uses the same version so local CI and the
# Cloudflare build stay in sync.

set -euo pipefail

MDBOOK_VERSION="v0.5.3"
MDBOOK_URL="https://github.com/rust-lang/mdBook/releases/download/${MDBOOK_VERSION}/mdbook-${MDBOOK_VERSION}-x86_64-unknown-linux-gnu.tar.gz"

echo "Downloading mdBook ${MDBOOK_VERSION}..."
mkdir -p ./.cloudflare/bin
curl -sSL "${MDBOOK_URL}" | tar -xz -C ./.cloudflare/bin

# Stamp the version footer (assets/js/version.js) with the library version and
# the docs' last-updated date before building. Version comes from the root
# Cargo.toml [workspace.package] (the single source of truth the CI version
# lint enforces); the date is the commit date of the most recent docs/ change,
# so a rebuild of the same commit produces the same output.
#
# The committed version.js holds placeholders; substitution happens on a build
# copy so the source file stays pristine and the working tree stays clean.
VERSION=$(grep -m1 '^version = ' Cargo.toml | cut -d'"' -f2)
UPDATED=$(git log -1 --format='%cs' -- docs/ 2>/dev/null || echo "")
echo "Stamping docs footer: version=${VERSION} updated=${UPDATED}"
cp assets/js/version.js assets/js/version.js.orig
sed -i \
  -e "s/__HONEYEATER_VERSION__/${VERSION}/g" \
  -e "s/__HONEYEATER_UPDATED__/${UPDATED}/g" \
  assets/js/version.js

echo "Building the book..."
./.cloudflare/bin/mdbook build

# Restore the pristine placeholder source so the working tree is unchanged.
mv assets/js/version.js.orig assets/js/version.js

echo "Done. Output is in ./book/."
