#!/bin/sh
# Runs the built plug-ins in a real VectorCraft host (vectorcraft-cli, headless) and writes
# tests/host.png. Usage: VECTORCRAFT_CLI=/path/to/vectorcraft-cli tests/host.sh
# VECTORCRAFT_NO_PREFS keeps the modules out of the app's own Plug-ins folder.
set -e
cd "$(dirname "$0")/.."
CLI="${VECTORCRAFT_CLI:-vectorcraft-cli}"
"$CLI" --version
G=io.github.pompiedom.geomancer
STEPS=$(mktemp)
trap 'rm -f "$STEPS"' EXIT
cat > "$STEPS" <<JSON
[{"command": "plugin.install", "params": {"path": "$PWD/dist/geomancer.wasm"}},
 {"command": "plugin.install", "params": {"path": "$PWD/dist/geomancer-rosette.wasm"}},
 {"command": "plugin.$G.golden", "params": {"width": 420}},
 {"command": "select.none", "params": {}},
 {"command": "plugin.$G.spiral", "params": {"fermat": true, "turns": 2, "outerRadius": 60}},
 {"command": "select.none", "params": {}},
 {"command": "plugin.$G.wave", "params": {"form": "triangle", "length": 420, "amplitude": 12, "cycles": 12}},
 {"command": "object.move", "params": {"dx": 0, "dy": 180}},
 {"command": "select.none", "params": {}},
 {"command": "plugin.run", "params": {"id": "$G", "params": {"shape": "Rose", "rose_n": 7, "rose_d": 3, "rose_radius": 50}}},
 {"command": "object.move", "params": {"dx": 0, "dy": -190}},
 {"command": "shape.ellipse", "params": {"x": 120, "y": 560, "width": 16, "height": 40}},
 {"command": "effect.apply", "params": {"effect": "plugin.$G.rosette", "params": {"copies": 12, "mirror": true}}},
 {"command": "plugin.info", "params": {"id": "$G.rosette"}}]
JSON
VECTORCRAFT_NO_PREFS=1 "$CLI" run --commands "$STEPS" --export tests/host.png --scale 1
