# Node editor

Double-click a `.kmat`, `.kshader` or `.kfx` file in the Project panel to open the node editor.
The editor saves the graph automatically and recompiles the shader at once.

| File | What it is | Output node |
|---|---|---|
| `.kmat` | PBR material | **Output**: Base color, Roughness, Metallic, Emission, Alpha, Occlusion, Normal, Height |
| `.kshader` | Object shader (unlit colour, glow, vertex offset) | **Shader output** |
| `.kfx` | Full-screen camera effect | **Effect output** |

## The basics

- **Right-click** the canvas or press **Shift+A** to add a node. Categories on the left, nodes with descriptions on the right; the arrow keys and Enter work.
- **Drag** a node's output into empty space - the add menu opens and the new node is connected at once.
- Any output can feed any input: a number is spread over three channels.
- Wheel - zoom, MMB or Space+drag - pan, **F** - show everything.
- Ctrl+C / V / D - copy, paste, duplicate. Ctrl+Z / Y - undo, redo. Del - delete.
- A **Comment** is a frame with a note; nodes inside it move together with it.
- On the right is a live preview on a sphere, a cube or your own model. "Preview only the selected node" shows the result of the selected node.

## Previews inside nodes

Noise and pattern nodes show a small preview in the node body, so you see what you are tuning.
Right-click a node → **Show preview in node** enables the preview on any node. It refreshes a quarter of a second after an edit so sliders stay smooth.

## Categories

| Category | What is inside |
|---|---|
| **Input** | constants, textures, time, triplanar texture |
| **Geometry** | position, normal, height, UV, direction masks, faces (top / side / bottom), vertex offsets |
| **Light** | light angle, highlights, rim light, shadows, sky light, glow and pulsing glow |
| **Normals** | normal maps, bump from height, height to normal, normal blend and strength |
| **Noise** | fractal, Perlin, Voronoi cells, cracks, clouds, grunge, scratches |
| **Patterns** | checker, bricks, hexagons, planks, weave, grid, gradients, shapes |
| **Colour** | Color ramp, blend modes, HSV, colour correction, curves |
| **Math, Vectors** | arithmetic, curves, rotations, dot and cross products |

The full list is in the [node reference](./reference), the shipped materials in the [catalogue](./materials).

## Recipes

### A normal map from a file

1. In the right panel pick a PNG normal map for the **normal** slot.
2. Add a **Texture** node with slot `1` and connect it to **Normal map** (input Map).
3. Connect **Normal map** to **Output → Normal**. The *strength* slider controls the depth.

### Normals from a height map

- From a texture: **Height to normal** (texture slot, strength) → **Normal map** → **Output → Normal**.
- Directly from any height node (noise, pattern): connect it to **Output → Height** or to **Bump from height** - the material becomes bumpy with no textures.
- Details on top of a base normal: **Normal blend**.

### Rust and wear on the edges

1. **Fractal noise** → **Color ramp** (dark → orange → light).
2. **Fresnel** or **Direction mask** as the mask, **Blend colours** (mode *multiply*) mixes colour and mask.
3. Send the result to **Base color** and the same noise through **Map range** to **Roughness**.

### Snow on top, rock on the sides

**Surface sides** outputs red = top, green = side, blue = bottom. The **Red** node extracts the top, **Mix** blends rock and snow by that mask.
Add noise through **Height blend** to make the border ragged.

### Glowing edges

Connect **Rim light** (colour, power, intensity) to **Emission**. For a pulse use **Pulsing glow**. The glow feeds the Bloom of post-processing when it is on.

### A flat, hand-painted (stylized) look

1. **Fractal noise** → **Posterize** (3-5 steps) → **Color ramp**.
2. Keep **Roughness** high (0.85) and do not connect a normal: flat colour bands read as painted.
3. For a cel edge add **Fresnel** → **Greater than** → **Emission**.

The ready examples are in the `Stylized` folder of the material library (Toon Grass, Toon Stone, Candy Stripes, Comic Halftone...).

### Wind and waves (vertices)

In a `.kshader` graph connect **Wind sway** or **Wave offset** to **Shader output → Vertex offset**.
Nodes that sample textures or use derivatives (Texture, Bump, Shadow...) are unavailable in the vertex stage - the editor tells you.

## Color ramp

Four colour stops with positions 0..1. The gradient and the stop markers are drawn above the node.
Feed a 0..1 value (noise, Fresnel, height) into **Position** to colour it along the ramp.

## Blend modes

**Blend colours** takes the mode as a number: `0` mix, `1` multiply, `2` screen, `3` overlay, `4` add, `5` subtract, `6` difference,
`7` darken, `8` lighten, `9` soft light, `10` dodge, `11` burn, `12` hard light. Hover a parameter name to see its legend.

## Export to WGSL

**Export as WGSL shader** writes the generated shader to `assets/shaders/` so you can keep editing it as code.
You can come back from such a file to the graph (**Open node graph**).
