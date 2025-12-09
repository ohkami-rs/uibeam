#!/bin/bash
set -euo pipefail

ESBUILD_VERSION="0.27.0"

if [ $(basename "$PWD") != "runtime" ]; then
    echo "This script must be run from the 'runtime' directory"
    exit 1
fi

if [ ! -f "./esbuild" ]; then
    echo "Downloading esbuild v$ESBUILD_VERSION..."
    curl -fsSL "https://esbuild.github.io/dl/v$ESBUILD_VERSION" | sh
fi

./esbuild runtime.js \
    --bundle \
    --minify \
    --format=esm \
    --external:'/.uibeam/hydrate.js' \
    --outfile='../runtime.mjs'
