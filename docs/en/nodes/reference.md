---
outline: [2, 2]
---

# Node reference

This page is generated from the node library (`cargo run -p kvas_engine --example gen_node_docs`). Nodes in total: **190**.

The letters show where a node can be used: **M** - material (`.kmat`), **S** - object shader (`.kshader`), **E** - camera effect (`.kfx`).

Parameters are shown as `name (min..max)`. The descriptions are the tooltips of the editor (some older nodes only have Russian or no text yet).

## Input

Constants, textures, time, UV.

| Node | Where | Inputs | Parameters | Description |
|---|---|---|---|---|
| **Color** | MSE | - | colour | A constant colour. |
| **Value** | MSE | - | value (-100..100) | A constant number. |
| **Texture** | MS | UV | slot (0..2), tiling (0.01..100) | Samples one of the material images (slot 0 albedo, 1 normal, 2 ORM). |
| **Time** | MSE | - | - | Seconds since the game started. |
| **Object colour** | MS | - | - | The colour set in the MeshRenderer of the object. |
| **Triplanar texture** | MS | - | slot (0..2), scale (0.01..50), sharpness (1..32) | Projects a material image from three world axes, no stretching on any slope. |

## Geometry

Position, normal, height, direction masks, vertex offsets.

| Node | Where | Inputs | Parameters | Description |
|---|---|---|---|---|
| **UV** | MSE | - | - | Texture coordinates of the surface (0..1). |
| **World position** | MS | - | - | Position of the pixel in the world. |
| **Normal** | MS | - | - | Direction the surface faces. |
| **Height** | MS | - | - | World height (Y) of the pixel. |
| **Slope mask** | MS | - | from (0..1), to (0..1) | 1 on flat ground, 0 on steep slopes (between the two thresholds). |
| **Camera distance** | MS | - | range (1..1000) | 0 at the camera, 1 at the chosen range (fog, level of detail). |
| **View direction** | MS | - | - | Unit vector from the surface to the camera. |
| **Reflection** | MS | - | - | The view direction mirrored by the surface normal. |
| **Camera position** | MS | - | - | World position of the camera. |
| **Object position** | MS | - | - | World position of the object's origin. |
| **Object random** | MS | - | - | A random number that is different for every object but constant over time. |
| **Screen position** | MS | - | - | Pixel position on the screen (0..1). |
| **Direction mask** | MS | - | x (-1..1), y (-1..1), z (-1..1), threshold (-1..1), softness (0.001..2) | White where the surface faces the chosen direction (x, y, z). Top = (0,1,0). |
| **Surface sides** | MS | - | - | Red = top faces, green = side faces, blue = bottom faces. Split with Red / Green / Blue nodes. |
| **Triplanar weights** | MS | - | sharpness (1..32) | How strongly each world axis projection (X, Y, Z) applies on this surface. |
| **Local position** | MS | - | - | Position relative to the object's origin. |
| **Object distance** | MS | - | range (0.1..500) | 0 at the object's centre, 1 at the chosen range. |
| **World gradient** | MS | - | x (-1..1), y (-1..1), z (-1..1), from (-1000..1000), to (-1000..1000) | A 0..1 ramp along a world direction between two values (fog by height, snow line). |
| **Planar UV** | MS | - | axis (0..2), scale (0.01..100) | Texture coordinates projected from world space along an axis. |
| **Normal UV** | MS | - | - | Spherical coordinates from the normal (for sky-like mapping). |
| **Wave offset** | MS | - | height (0..5), frequency (0..30), speed (-10..10) | Vertex offset for waves. Connect to Shader output > Vertex offset. |
| **Wind sway** | MS | - | strength (0..3), speed (0..20), scale (0..10) | Vertex offset that bends things in the wind, more at the top. Connect to Vertex offset. |

## Light

Sun, shadows, highlights, rim light, glow (Emission).

| Node | Where | Inputs | Parameters | Description |
|---|---|---|---|---|
| **Fresnel** | MS | Power | - | Brighter at grazing angles (rim light, glass, water edges). |
| **Sun light** | MS | - | - | How much the surface faces the sun (0..1), without shadows. |
| **Sky reflection** | MS | - | - | What the sky looks like in the mirror direction. |
| **Sun direction** | MS | - | - | Unit vector pointing to the sun. |
| **Sun colour** | MS | - | - | Colour and strength of the sun. |
| **Ambient light** | MS | - | - | Constant ambient light colour of the scene. |
| **Shadow** | MS | - | - | 1 in light, 0 in the sun shadow. |
| **Lighting** | MS | Base | roughness (0..1) | Applies scene lighting (sun, shadows, lamps) to a colour. |
| **Light angle** | MS | - | - | Dot product of the normal and the sun direction: 1 facing the sun, -1 facing away. |
| **Half lambert** | MS | - | power (0.1..4) | Soft wrap-around sun lighting (0..1) without a hard dark side. |
| **Specular highlight** | MS | - | shininess (1..512), intensity (0..10) | Blinn-Phong glint of the sun. Higher shininess = smaller, sharper spot. |
| **Rim light** | MS | - | colour, power (0.1..12), intensity (0..20) | A coloured glow on the edges that face away from the camera. Plug into Emission. |
| **Backlight** | MS | - | sharpness (1..64) | Bright when the camera looks towards the sun through the surface (leaves, cloth, ice). |
| **Facing ratio** | MS | - | - | 1 where the surface faces the camera, 0 on silhouettes. |
| **Sun with shadow** | MS | - | - | Sun lighting amount (0..1) that respects shadows. |
| **Sun lighting** | MS | - | - | Sun colour times the lighting amount, with shadows. Add Sky light for a full diffuse term. |
| **Sky light** | MS | - | - | Ambient light from the sky in the direction of the normal. |
| **Glow** | MSE | - | colour, intensity (0..50) | A constant colour multiplied by an intensity. Plug into Emission to make things shine. |
| **Pulsing glow** | MSE | - | colour, intensity (0..50), speed (0..20) | A glow that pulses over time. |

## Normals

Normal maps, bump from height, normal blending.

| Node | Where | Inputs | Parameters | Description |
|---|---|---|---|---|
| **Normal map** | MS | Map, UV | strength (0..4) | Turns a normal-map colour into a surface direction. Connect the result to Output > Normal. |
| **Bump from height** | MS | Height | strength (0..10) | Makes bumps from a height value. Connect the result to Output > Normal. |
| **Normal strength** | MS | Normal | strength (0..4) | Blends between the surface normal and the connected normal. |
| **Normal blend** | MS | Base, Detail | - | Adds a detail normal on top of a base normal. |
| **Normal to colour** | MS | Normal | - | Shows a normal as a colour (for previews and debugging). |
| **Noise bump** | MS | Coords | scale (0.05..100), strength (0..10) | Bumps the surface with fractal noise. Connect to Output > Normal. |
| **Height to normal** | MS | UV | slot (0..2), tiling (0.01..100), strength (0..32) | Makes a normal-map colour from the height in a texture slot. Feed it to Normal map. |

## Noise

Fractal, Perlin, Voronoi, cracks, clouds, grunge. They show a preview by default.

| Node | Where | Inputs | Parameters | Description |
|---|---|---|---|---|
| **Noise** | MSE | Coords | scale (0.01..100) | Simple smooth value noise (2D, uses the X and Z of the coordinates). |
| **Voronoi** | MSE | Coords | scale (0.01..100) | Cell pattern: distance to the nearest random point. |
| **Cell colour** | MSE | Coords | scale (0.01..100) | A random colour per Voronoi cell. |
| **Noise 3D** | MS | Coords | scale (0.01..100) | Smooth 3D noise that does not stretch on any surface. |
| **Ridged noise** | MS | Coords | scale (0.01..100) | Noise with sharp ridges (mountains, veins). |
| **Turbulence** | MS | Coords | scale (0.01..100) | Billowy noise with creases (smoke, fire). |
| **White noise** | MSE | Coords | cells (1..1000) | A random grey value per cell, no smoothing (TV static). |
| **Sparkle** | MSE | Coords | cells (4..1000), density (0..1) | Rare bright dots (stars, glitter). |
| **Fractal noise** | MSE | Coords | scale (0.01..100), octaves (1..8), lacunarity (1.2..4), gain (0.1..0.9), type (0..2) | Layered noise with control over octaves, lacunarity and gain. Types: fBm, ridged, billow. |
| **Perlin noise** | MSE | Coords | scale (0.01..100) | Smooth gradient noise (0..1). |
| **Voronoi cells** | MSE | Coords | scale (0.01..100), randomness (0..1), output (0..3) | 3D cell noise. Outputs: distance, cell edges, random id per cell, second distance. |
| **Cracks** | MSE | Coords | scale (0.01..100), width (0.005..0.5) | Thin cracks along cell borders (dry ground, ice, shattered stone). |
| **Warped noise** | MSE | Coords | scale (0.01..100), warp (0..8) | Noise bent by other noise - swirly, organic shapes. |
| **Clouds** | MSE | Coords | scale (0.01..100), coverage (0..1), softness (0.01..1) | Soft cloud-like blobs. Coverage sets how much is filled. |
| **Grunge** | MSE | Coords | scale (0.01..100), contrast (0.1..8) | Dirt and wear mask. Multiply with a base colour or use for roughness. |
| **Scratches** | MSE | Coords | density (2..400), angle (-3.15..3.15) | Thin stretched scratches at an angle. |

## Patterns

Checker, bricks, hexagons, planks, weave, gradients and shapes.

| Node | Where | Inputs | Parameters | Description |
|---|---|---|---|---|
| **Checker** | MSE | Coords | scale (1..200) | A black and white checkerboard. |
| **Stripes** | MSE | Coords | scale (1..200), sharpness (0..1) | Parallel stripes along X. |
| **Bricks** | MSE | Coords | scale (1..100), mortar (0..0.3) | A brick wall pattern with mortar lines. |
| **Dots** | MSE | Coords | scale (1..200), size (0.01..0.5) | A grid of round dots. |
| **Caustics** | MS | Coords | scale (0.05..50), speed (0..10) | Moving light patterns like under water. |
| **Hexagons** | MSE | Coords | scale (0.5..100), edge width (0..0.5) | Honeycomb pattern edges. |
| **Wave rings** | MSE | Coords | frequency (1..200), speed (-20..20) | Animated rings spreading from the centre. |
| **Linear gradient** | MSE | Coords | - | 0..1 from left to right (X of the coordinates). |
| **Radial gradient** | MSE | Coords | - | 1 in the centre, 0 at the edge. |
| **Angular gradient** | MSE | Coords | - | 0..1 around the centre like a clock hand. |
| **Circle** | MSE | Coords | radius (0..1), softness (0..0.5) | A soft disc. |
| **Box** | MSE | Coords | width (0..1), height (0..1), softness (0..0.5) | A soft rectangle. |
| **Wood rings** | MS | Coords | scale (0.1..50), wobble (0..2) | Tree-ring pattern for wood. |
| **Marble veins** | MS | Coords | scale (0.05..20), warp (0..20) | Wavy veins for marble. |
| **Grid lines** | MSE | Coords | cells (1..200), line width (0.001..0.5) | A grid of thin lines (white lines on black). |
| **Planks** | MSE | Coords | rows (1..100), plank length (0.2..40), gap (0..0.3) | Wooden floor boards with random tones and gaps. |
| **Weave** | MSE | Coords | threads (1..200) | Over-under fabric weave pattern. |

## Colour

Color ramp, blend modes, HSV, colour correction.

| Node | Where | Inputs | Parameters | Description |
|---|---|---|---|---|
| **Hue shift** | MSE | Colour | turns (-1..1) | Rotates the colours around the colour wheel. |
| **Saturation** | MSE | Colour | amount (0..3) | 0 = grey, 1 = unchanged, above 1 = more vivid. |
| **Contrast** | MSE | Colour | amount (0..3) | Pushes values away from mid grey. |
| **Brightness** | MSE | Colour | scale (0..10) | Multiplies the colour. |
| **Gradient** | MSE | Position, From, To | - | Blends two colours by a 0..1 position. |
| **To grey** | MSE | Colour | - | Converts a colour to its brightness. |
| **Overlay** | MSE | Base, Blend | - | Overlay blend: multiplies darks and screens lights. |
| **Screen blend** | MSE | Base, Blend | - | Screen blend: always lightens. |
| **Colourize** | MSE | Value, Tint | - | Tints the brightness of a colour with another colour. |
| **Ramp 3 colours** | MSE | Position, Low, Middle, High | - | Maps 0..1 to a three-colour gradient. |
| **Gamma** | MSE | Colour | gamma (0.1..5) | Gamma correction. |
| **Exposure** | MSE | Colour | stops (-8..8) | Brightens or darkens by photographic stops. |
| **Levels** | MSE | Colour | black (0..1), white (0..2) | Sets the black and white points. |
| **Threshold** | MSE | Colour | level (0..1) | Black and white by a brightness threshold. |
| **Tone map** | MSE | Colour | - | Filmic (ACES) tone mapping of HDR colour. |
| **Color ramp** | MSE | Position, Stop 1, Stop 2, Stop 3, Stop 4 | position 1 (0..1), position 2 (0..1), position 3 (0..1), position 4 (0..1) | Maps 0..1 to a gradient of four colour stops placed at four positions. |
| **Blend colours** | MSE | A, B, Factor | mode (0..12) | Photoshop-style blend modes: 0 mix, 1 multiply, 2 screen, 3 overlay, 4 add, 5 subtract, 6 difference, 7 darken, 8 lighten, 9 soft light, 10 dodge, 11 burn, 12 hard light. |
| **HSV adjust** | MSE | Colour | hue shift (-0.5..0.5), saturation (0..3), value (0..3) | Shifts hue and scales saturation and brightness. |
| **HSV to RGB** | MSE | Hue, Saturation, Value | - | Builds a colour from hue, saturation and value. |
| **RGB to HSV** | MSE | Colour | - | Splits a colour into hue (red), saturation (green), value (blue). |
| **Lift gamma gain** | MSE | Colour | lift (-0.5..0.5), gamma (0.1..4), gain (0..4) | Classic three-way colour correction. |
| **Colour temperature** | MSE | Colour | warmth (-1..1) | Warm (orange) or cool (blue) tint. |
| **Colour key** | MSE | Colour | colour, tolerance (0..1.75), softness (0..1) | White where the colour is close to the key colour. |
| **S-curve** | MSE | Value | amount (0.05..0.95) | Contrast curve that keeps black and white fixed. |
| **Bias curve** | MSE | Value | bias (0.01..0.99) | Bends mid-tones towards dark or light. |

## Math

Arithmetic, curves, comparisons, transitions.

| Node | Where | Inputs | Parameters | Description |
|---|---|---|---|---|
| **Add** | MSE | A, B | - | A + B. |
| **Subtract** | MSE | A, B | - | A - B. |
| **Multiply** | MSE | A, B | - | A * B (tinting, masking). |
| **Divide** | MSE | A, B | - | A / B (safe: never divides by zero). |
| **Mix** | MSE | A, B, Factor | - | Blends A and B by Factor (0 = A, 1 = B). |
| **Power** | MSE | Value, Exponent | - | Raises to a power (curves, sharpening masks). |
| **Clamp** | MSE | Value | min (-100..100), max (-100..100) | Limits a value to a range. |
| **One minus** | MSE | Value | - | 1 - value (inverts a mask). |
| **Sine** | MSE | Value | frequency (0..100) | A sine wave remapped to 0..1. |
| **Smoothstep** | MSE | Value | edge 0 (-10..10), edge 1 (-10..10) | Smooth 0..1 transition between two edges. |
| **Abs** | MSE | Value | - | Absolute value. |
| **Min** | MSE | A, B | - | The smaller of two values. |
| **Max** | MSE | A, B | - | The larger of two values. |
| **Fract** | MSE | Value | - | The fractional part (repeats 0..1). |
| **Step** | MSE | Value | edge (0..1) | 0 below the edge, 1 above it. |
| **Remap** | MSE | Value | in min (-10..10), in max (-10..10), out min (-10..10), out max (-10..10) | Maps a range to another range (not clamped). |
| **Negate** | MSE | Value | - | Changes the sign. |
| **Square root** | MSE | Value | - | Square root. |
| **Floor** | MSE | Value | - | Rounds down. |
| **Ceil** | MSE | Value | - | Rounds up. |
| **Round** | MSE | Value | - | Rounds to the nearest whole number. |
| **Sign** | MSE | Value | - | -1, 0 or 1. |
| **Saturate** | MSE | Value | - | Clamps to 0..1. |
| **Modulo** | MSE | A, B | - | The remainder of A / B (repeating). |
| **Multiply add** | MSE | A, B, C | - | A * B + C in one node. |
| **Cosine** | MSE | Value | frequency (0..100) | A cosine wave remapped to 0..1. |
| **Triangle wave** | MSE | Value | frequency (0..100) | A zig-zag wave 0..1..0. |
| **Sawtooth** | MSE | Value | frequency (0..100) | A ramp that jumps back to 0. |
| **Square wave** | MSE | Value | frequency (0..100) | Alternates between 0 and 1. |
| **Posterize** | MSE | Value | steps (2..64) | Reduces the number of distinct levels. |
| **Greater than** | MSE | A, B | - | 1 where A > B, otherwise 0. |
| **Branch** | MSE | A, B, If A > B, Otherwise | - | If A > B returns the third input, otherwise the fourth. |
| **Height blend** | MSE | Height A, Height B, Blend | sharpness (0.001..2) | A blend mask where the higher of two heights wins (rock through snow, moss in cracks). |
| **Exponential** | MSE | Value | - | e raised to the value. |
| **Logarithm** | MSE | Value | - | Natural logarithm. |
| **Reciprocal** | MSE | Value | - | 1 divided by the value. |
| **Inverse lerp** | MSE | Value | from (-100..100), to (-100..100) | Where the value lies between from and to (0..1 inside). |
| **Map range** | MSE | Value | in min (-100..100), in max (-100..100), out min (-100..100), out max (-100..100) | Remaps a range to another and clamps the result. |
| **Snap** | MSE | Value | step (0.0001..100) | Rounds to the nearest multiple of the step. |
| **Arc tangent 2** | MSE | Y, X | - | Angle of the point (X, Y) in radians. |
| **Smootherstep** | MSE | Value | edge 0 (-10..10), edge 1 (-10..10) | A smoother version of Smoothstep. |
| **Less than** | MSE | A, B | - | 1 where A < B, otherwise 0. |
| **Average** | MSE | A, B | - | Mean of two values. |

## Vectors

Components, dot and cross products, rotations, coordinates.

| Node | Where | Inputs | Parameters | Description |
|---|---|---|---|---|
| **Red** | MSE | Vector | - | Copies the red channel to all channels. |
| **Green** | MSE | Vector | - | Copies the green channel to all channels. |
| **Blue** | MSE | Vector | - | Copies the blue channel to all channels. |
| **Combine** | MSE | R, G, B | - | Builds a vector from the red channels of three inputs. |
| **Length** | MSE | Vector | - | Length of a vector. |
| **Distance** | MSE | A, B | - | Distance between two points. |
| **Dot** | MSE | A, B | - | Dot product (how much two directions agree). |
| **Cross** | MSE | A, B | - | Cross product (a direction perpendicular to both). |
| **Normalize** | MSE | Vector | - | Scales a vector to length 1. |
| **Reflect** | MSE | Incident, Normal | - | Mirrors a direction around a normal. |
| **Rotate coords** | MSE | Coords | angle (-6.3..6.3) | Rotates texture coordinates around the centre. |
| **Scale offset coords** | MSE | Coords | scale x (0.01..100), scale y (0.01..100), offset x (-100..100), offset y (-100..100) | Tiles and shifts texture coordinates. |
| **Scroll coords** | MSE | Coords | speed x (-10..10), speed y (-10..10) | Moves coordinates over time (flowing water, conveyor belts). |
| **Polar coords** | MSE | Coords | - | Converts coordinates to angle and radius. |
| **Mirror tile** | MSE | Coords | tiles (0.1..50) | Repeats coordinates mirrored, so the seams disappear. |
| **Flipbook** | MSE | Coords | columns (1..32), rows (1..32), frames per second (0..60) | Plays a grid of frames (sprite sheet) as animation. |
| **Distort coords** | MSE | Coords, Amount | scale (0.1..100) | Wobbles coordinates with noise (heat haze, water). |
| **Rotate around axis** | MSE | Vector, Axis | angle (-6.3..6.3) | Rotates a vector around an axis by an angle (radians). |
| **Refract** | MSE | Incident, Normal | index ratio (0.1..2) | Bends a direction through a surface (glass, water). |
| **Twirl coords** | MSE | Coords | amount (-20..20) | Swirls coordinates around the centre. |

## Utility

Reroute, comments, number from the Inspector.

| Node | Where | Inputs | Parameters | Description |
|---|---|---|---|---|
| **Reroute** | MSE | Value | - | A tiny pass-through point to tidy long links. |
| **Comment** | MSE | - | - | A frame with a note. Nodes inside it move with it. |
| **Inspector number** | MSE | - | slot 0-7 (0..7), default (-1000..1000), zero uses default (0..1) | A number you can change per object in the Inspector (MeshRenderer user 1..8) or from a script. |
| **Time scaled** | MSE | - | speed (-50..50) | Time multiplied by a speed. |
| **Pulse** | MSE | - | hertz (0..20) | A smooth 0..1 wave over time. |

## Camera

Only in camera effect graphs (.kfx).

| Node | Where | Inputs | Parameters | Description |
|---|---|---|---|---|
| **Screen color** | E | Coords | - | Colour of the camera image at the given coordinates (blur, distortion). |
| **Pixel colour** | E | - | - | The current pixel of the camera image. |
| **Pixel size** | E | - | - | Size of one screen pixel in 0..1 units. |
| **Aspect ratio** | E | - | - | Screen width divided by height. |
| **Blur** | E | Coords | radius px (0..30) | Cheap 5-tap blur of the camera image. |
| **Chromatic split** | E | Coords | amount px (0..30) | Shifts the red and blue channels apart. |
| **Edge detect** | E | Coords | strength (0..20) | White where the picture changes sharply. |
| **Pixelate** | E | Coords | cells across (8..1000) | Blocky retro look. |
| **Vignette** | E | Coords | strength (0..2) | Darkens the screen edges. |
| **Scanlines** | E | Coords | lines (10..2000) | CRT-like horizontal lines. |

## Output nodes

| Graph | Node | Inputs |
|---|---|---|
| Material | **Output** | Base color, Roughness, Metallic, Emission, Alpha, Occlusion, Normal, Height |
| Object shader | **Shader output** | Color, Alpha, Emission, Light amount, Roughness, Vertex offset |
| Camera effect | **Effect output** | Color |
