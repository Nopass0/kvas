#!/usr/bin/env bash
# Rebuilds the scripts of the demo and test projects and copies them where `kvas_player --project` looks.
set -e
cd "$(dirname "$0")/.."
cargo build -q -p demo_scripts -p test_meadow_scripts -p test_voxels_scripts -p test_cloth_scripts -p test_assets_scripts -p test_fps_scripts
for p in demo test_meadow test_voxels test_cloth test_assets test_fps; do
  lib="$p"; [ "$p" = demo ] && lib=demo
  mkdir -p "games/$p/.kvas"
  cp "target/debug/${lib}_scripts.dll" "games/$p/.kvas/" 2>/dev/null || cp "target/debug/${p}_scripts.dll" "games/$p/.kvas/"
done
echo "script libraries refreshed"
