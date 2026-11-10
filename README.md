# Geomancer

Geometry tools for [VectorCraft](https://github.com/storytold/vectorcraft): spirals, rose curves, waves,
golden rectangles and rosettes, as sandboxed WebAssembly plug-ins (VectorCraft plug-in ABI v1, see its
`docs/plugins.md`). Two `.wasm` files, running on macOS, Windows, Linux and the web build. Needs VectorCraft
0.23 or later (plug-in commands); tested with 0.23.0.

| Plug-in | File | Where in VectorCraft |
|---|---|---|
| Geomancer | `geomancer.wasm` (object filter) | Object › Plug-ins › Geomancer |
| Rosette | `geomancer-rosette.wasm` (live effect) | Effect › Plug-ins › Rosette |

**Geomancer** is a submenu with a command per shape, each with its own dialog and live preview; Edit ›
Keyboard Shortcuts can give each one a shortcut. **Geomancer…** at the top of the submenu opens one dialog
with a shape picker instead.

| Command | Parameters |
|---|---|
| Spiral… | inner/outer radius, turns, Fermat (two-armed spiral with an S through the centre), direction, start angle |
| Rose… | radius, n, d (r = R·cos(n/d·θ)), rotation, fill |
| Wave… | sine, triangle, square or sawtooth; length, amplitude, cycles, phase |
| Golden Rectangle… | width, landscape or portrait, number of squares, direction, golden spiral (a path of its own) |

Each shape is placed at the centre of the selection, or of the artboard when nothing is selected, with a 1 pt
black stroke (Rose: a black fill when *fill* is on); the selection is left alone. Agents and actions call the
commands as `plugin.io.github.pompiedom.geomancer.<spiral|rose|wave|golden>`.

**Rosette** repeats an object around a centre point and stays live: edit the original and the rosette follows.
Options: copies, mirror (dihedral symmetry), centre (bottom/top/left/right/middle of the object) + offset.

None of them asks for permissions when installed: they list none and get what their kind implies.

VectorCraft has no tool plug-ins yet, so these can't be dragged out on the canvas or placed in the toolbar
(a proposal is open as storytold/vectorcraft#5654). Plug-in dialogs open at their defaults each time; the
host doesn't hand them the values used last.

### Upgrading from 0.1

0.1 had Spiral and Rose as two separate plug-ins. Remove them (`plugin.remove` with
`io.github.pompiedom.geomancer.spiral` and `….rose`, or delete them from the Plug-ins folder) and install
`geomancer.wasm`. Rosette keeps its id, so documents that use it are unaffected.

## Install

Download the two `.wasm` files from `dist/` (or build them), then in VectorCraft either:

- **File › Open** each `.wasm`, or
- set Edit › Preferences › Performance & Storage › *Additional Plug-ins Folder* to the folder holding them.

## Build and test

Needs Rust with the `wasm32-unknown-unknown` target, and Node.js for the tests.

```sh
rustup target add wasm32-unknown-unknown
./build.sh            # builds and copies the modules to dist/
node tests/run.mjs    # calls them the way VectorCraft's host does and writes tests/preview.html
VECTORCRAFT_CLI=/path/to/vectorcraft-cli tests/host.sh   # runs them in a real (headless) VectorCraft, writes tests/host.png
```

`common/` holds the ABI exports (the `plugin!` macro), the curve-to-Bézier sampler and the placement of
generated shapes. `generators/` is the Geomancer filter, a module per shape; `rosette/` the live effect.

## Planned

Spike, Rhombus, Mirror, Tile (the 17 wallpaper groups), Cut and Rectangular Cut. Grid and Polar Mesh are
dropped: VectorCraft has Rectangular Grid and Polar Grid tools of its own.

## Licence

MIT or Apache-2.0, at your option.
