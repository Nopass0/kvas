---
outline: [2, 3]
---

# Camera

Only in camera effect graphs (.kfx).

Generated from the node library. Socket types: **number** (grey circle), **colour** (yellow circle), **vector** (purple diamond). Any output can feed any input; when the types differ, pick a conversion on the input (right-click it). See [how nodes connect](../wiring).

## Screen color

Colour of the camera image at the given coordinates (blur, distortion).

*Works in: camera effect*

**Inputs**

| Input | Type | When not connected |
|---|---|---|
| Coords | vector | the object's UV |

**Output**: number

> Use it as a mask: into **Factor** of Mix colour, **Position** of a Color ramp, **Roughness / Metallic / Alpha / Height** of the output, or **A/B** of Math.

Example: Texture coordinate (World) → Mapping (scale 4) → **Screen color** (input Coords) → Color ramp → Output > Base color.

## Pixel colour

The current pixel of the camera image.

*Works in: camera effect*

**Output**: number

> Use it as a mask: into **Factor** of Mix colour, **Position** of a Color ramp, **Roughness / Metallic / Alpha / Height** of the output, or **A/B** of Math.

## Pixel size

Size of one screen pixel in 0..1 units.

*Works in: camera effect*

**Output**: number

> Use it as a mask: into **Factor** of Mix colour, **Position** of a Color ramp, **Roughness / Metallic / Alpha / Height** of the output, or **A/B** of Math.

## Aspect ratio

Screen width divided by height.

*Works in: camera effect*

**Output**: number

> Use it as a mask: into **Factor** of Mix colour, **Position** of a Color ramp, **Roughness / Metallic / Alpha / Height** of the output, or **A/B** of Math.

## Blur

Cheap 5-tap blur of the camera image.

*Works in: camera effect*

**Inputs**

| Input | Type | When not connected |
|---|---|---|
| Coords | vector | the object's UV |

**Parameters** (on the node or in the Inspector)

| Parameter | Default | Range | Choices |
|---|---|---|---|
| radius px | 2 | 0..30 | - |

**Output**: number

> Use it as a mask: into **Factor** of Mix colour, **Position** of a Color ramp, **Roughness / Metallic / Alpha / Height** of the output, or **A/B** of Math.

Example: Texture coordinate (World) → Mapping (scale 4) → **Blur** (input Coords) → Color ramp → Output > Base color.

## Chromatic split

Shifts the red and blue channels apart.

*Works in: camera effect*

**Inputs**

| Input | Type | When not connected |
|---|---|---|
| Coords | vector | the object's UV |

**Parameters** (on the node or in the Inspector)

| Parameter | Default | Range | Choices |
|---|---|---|---|
| amount px | 3 | 0..30 | - |

**Output**: number

> Use it as a mask: into **Factor** of Mix colour, **Position** of a Color ramp, **Roughness / Metallic / Alpha / Height** of the output, or **A/B** of Math.

Example: Texture coordinate (World) → Mapping (scale 4) → **Chromatic split** (input Coords) → Color ramp → Output > Base color.

## Edge detect

White where the picture changes sharply.

*Works in: camera effect*

**Inputs**

| Input | Type | When not connected |
|---|---|---|
| Coords | vector | the object's UV |

**Parameters** (on the node or in the Inspector)

| Parameter | Default | Range | Choices |
|---|---|---|---|
| strength | 4 | 0..20 | - |

**Output**: number

> Use it as a mask: into **Factor** of Mix colour, **Position** of a Color ramp, **Roughness / Metallic / Alpha / Height** of the output, or **A/B** of Math.

Example: Texture coordinate (World) → Mapping (scale 4) → **Edge detect** (input Coords) → Color ramp → Output > Base color.

## Pixelate

Blocky retro look.

*Works in: camera effect*

**Inputs**

| Input | Type | When not connected |
|---|---|---|
| Coords | vector | the object's UV |

**Parameters** (on the node or in the Inspector)

| Parameter | Default | Range | Choices |
|---|---|---|---|
| cells across | 160 | 8..1000 | - |

**Output**: number

> Use it as a mask: into **Factor** of Mix colour, **Position** of a Color ramp, **Roughness / Metallic / Alpha / Height** of the output, or **A/B** of Math.

Example: Texture coordinate (World) → Mapping (scale 4) → **Pixelate** (input Coords) → Color ramp → Output > Base color.

## Vignette

Darkens the screen edges.

*Works in: camera effect*

**Inputs**

| Input | Type | When not connected |
|---|---|---|
| Coords | vector | the object's UV |

**Parameters** (on the node or in the Inspector)

| Parameter | Default | Range | Choices |
|---|---|---|---|
| strength | 0.4 | 0..2 | - |

**Output**: number

> Use it as a mask: into **Factor** of Mix colour, **Position** of a Color ramp, **Roughness / Metallic / Alpha / Height** of the output, or **A/B** of Math.

Example: Texture coordinate (World) → Mapping (scale 4) → **Vignette** (input Coords) → Color ramp → Output > Base color.

## Scanlines

CRT-like horizontal lines.

*Works in: camera effect*

**Inputs**

| Input | Type | When not connected |
|---|---|---|
| Coords | vector | the object's UV |

**Parameters** (on the node or in the Inspector)

| Parameter | Default | Range | Choices |
|---|---|---|---|
| lines | 300 | 10..2000 | - |

**Output**: number

> Use it as a mask: into **Factor** of Mix colour, **Position** of a Color ramp, **Roughness / Metallic / Alpha / Height** of the output, or **A/B** of Math.

Example: Texture coordinate (World) → Mapping (scale 4) → **Scanlines** (input Coords) → Color ramp → Output > Base color.

