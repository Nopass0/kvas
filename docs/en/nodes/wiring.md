# How nodes connect

Every socket carries three numbers (a vec3). A number is stored in all three channels, a colour as RGB, a vector as XYZ. So **any output connects to any input**; the type and the conversion give it meaning.

## Socket types

| Type | Mark | Examples |
|---|---|---|
| number | grey circle | Roughness, Factor, Scale, noise masks |
| colour | yellow circle | Base color, Colour, Stop 1..4 |
| vector | purple diamond | Vector, Coords, UV, Normal |

## Conversions on an input

Right-click an input (or use the list in the Inspector) to choose how the incoming value is turned before use. A badge shows it on the link. A colour plugged into a number input gets "Luminance" automatically.

| # | Conversion | Badge |
|---|---|---|
| 0 | Auto | `` |
| 1 | Red / X | `R` |
| 2 | Green / Y | `G` |
| 3 | Blue / Z | `B` |
| 4 | Luminance | `L` |
| 5 | Average | `Avg` |
| 6 | Length | `Len` |
| 7 | Largest | `Max` |
| 8 | Invert (1 - x) | `1-x` |
| 9 | Negate | `-x` |
| 10 | Clamp 0..1 | `0..1` |
| 11 | To 0..1 (x * 0.5 + 0.5) | `½` |
| 12 | To -1..1 (x * 2 - 1) | `±` |

## Several outputs

Image texture gives colour, R, G, B and luminance; Voronoi texture gives factor, distance, edge, cell value and cell colour; Texture coordinate gives UV, object, world, normal, reflection and screen; Brick texture gives colour, mask, brick value and distance to the mortar. Drag the link from the output you need; output 0 is the main one.

## Keyboard shortcuts

Shift+A or double-click add a node; Ctrl+C / V / D copy / paste / duplicate; Delete or X delete; Ctrl+G frame in a comment; Ctrl+L select everything upstream; Alt+D disconnect inputs; H previews inside nodes; P preview the selected node; arrows nudge; F fit; 1 zoom 100%; Ctrl+S save; Ctrl+E export WGSL; K the shortcut sheet.

## Example: marble from basic nodes

1. Texture coordinate (World) → Mapping (scale 3).
2. Vector math (Dot product) with (0.82, 0.31, 0.48) — the vein direction.
3. Noise texture (scale 0.9, detail 7, roughness 0.58) — turbulence.
4. Math (Multiply add): noise × 3 + direction → phase; Sine → Absolute — thin veins near zero.
5. Color ramp (Ease): vein colour at 0, base from 0.16.
6. Mix colour (Overlay) with a coloured noise for clouds; Map range of the veins → Roughness; Bump from the veins → Normal.

Ready graphs are in `materials/library/Game Ready/` — open any to see how it is built.
