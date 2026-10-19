# Geomancer

Geometry tools for [VectorCraft](https://github.com/storytold/vectorcraft): spirals, rose curves and
rosettes, as sandboxed WebAssembly plug-ins (VectorCraft plug-in ABI v1, see its `docs/plugins.md`).
One `.wasm` file per tool, running on macOS, Windows, Linux and the web build.

| Plug-in | Kind | Where in VectorCraft | Parameters |
|---|---|---|---|
| Spiral | object filter (generator) | Object › Plug-ins › Spiral | inner/outer radius, turns, Fermat (two-armed spiral with an S through the centre), direction, start angle |
| Rose | object filter (generator) | Object › Plug-ins › Rose | radius, n, d (r = R·cos(n/d·θ)), rotation, fill |
| Rosette | live effect | Effect › Plug-ins › Rosette | copies, mirror (dihedral symmetry), centre (bottom/top/left/right/middle of the object) + offset |

Spiral and Rose place their shape at the centre of the selection, or of the artboard when nothing is
selected, with a 1 pt black stroke (Rose: a black fill when *fill* is on); the selection is left alone.
Each opens a dialog with a live preview. Rosette repeats an object around a centre point and stays live:
edit the original and the rosette follows.

VectorCraft has no tool plug-ins yet, so these can't be dragged out on the canvas or placed in the toolbar.

## Install

Download the `.wasm` files from `dist/` (or build them), then in VectorCraft either:

- **File › Open** each `.wasm`, or
- set Edit › Preferences › Performance & Storage › *Additional Plug-ins Folder* to the folder holding them.

## Build and test

Needs Rust with the `wasm32-unknown-unknown` target, and Node.js for the tests.

```sh
rustup target add wasm32-unknown-unknown
./build.sh            # builds and copies the modules to dist/
node tests/run.mjs    # calls them the way VectorCraft's host does and writes tests/preview.html
```

`common/` holds the ABI exports (the `plugin!` macro), the curve-to-Bézier sampler and the placement of
generated shapes; each tool is a small crate (`spiral/`, `rose/`, `rosette/`).

## Planned

Wave, Spike, Rhombus, Golden Rectangle, Grid, Polar Mesh, Mirror, Tile (the 17 wallpaper groups), Cut
and Rectangular Cut.

## Licence

MIT or Apache-2.0, at your option.
