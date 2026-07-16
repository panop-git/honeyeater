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

echo "Building the book..."
./.cloudflare/bin/mdbook build

echo "Done. Output is in ./book/."
