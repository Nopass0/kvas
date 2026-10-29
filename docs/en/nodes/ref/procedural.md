---
outline: [2, 3]
---

# Patterns

Checker, bricks, hexagons, planks, weave, gradients and shapes.

Generated from the node library. Socket types: **number** (grey circle), **colour** (yellow circle), **vector** (purple diamond). Any output can feed any input; when the types differ, pick a conversion on the input (right-click it). See [how nodes connect](../wiring).

## Checker

A black and white checkerboard.

*Works in: material, object shader, camera effect*

**Inputs**

| Input | Type | When not connected |
|---|---|---|
| Coords | vector | the object's UV |

**Parameters** (on the node or in the Inspector)

| Parameter | Default | Range | Choices |
|---|---|---|---|
| scale | 8 | 1..200 | - |

**Outputs**

| Output | Type |
|---|---|
| Factor | number |
| Inverted | number |
| Sharp | number |

> Use it as a mask: into **Factor** of Mix colour, **Position** of a Color ramp, **Roughness / Metallic / Alpha / Height** of the output, or **A/B** of Math.

Example: Texture coordinate (World) → Mapping (scale 4) → **Checker** (input Coords) → Color ramp → Output > Base color.

## Stripes

Parallel stripes along X.

*Works in: material, object shader, camera effect*

**Inputs**

| Input | Type | When not connected |
|---|---|---|
| Coords | vector | the object's UV |

**Parameters** (on the node or in the Inspector)

| Parameter | Default | Range | Choices |
|---|---|---|---|
| scale | 10 | 1..200 | - |
| sharpness | 0.5 | 0..1 | - |

**Outputs**

| Output | Type |
|---|---|
| Factor | number |
| Inverted | number |
| Sharp | number |

> Use it as a mask: into **Factor** of Mix colour, **Position** of a Color ramp, **Roughness / Metallic / Alpha / Height** of the output, or **A/B** of Math.

Example: Texture coordinate (World) → Mapping (scale 4) → **Stripes** (input Coords) → Color ramp → Output > Base color.

## Bricks

A brick wall pattern with mortar lines.

*Works in: material, object shader, camera effect*

**Inputs**

| Input | Type | When not connected |
|---|---|---|
| Coords | vector | the object's UV |

**Parameters** (on the node or in the Inspector)

| Parameter | Default | Range | Choices |
|---|---|---|---|
| scale | 6 | 1..100 | - |
| mortar | 0.06 | 0..0.3 | - |

**Outputs**

| Output | Type |
|---|---|
| Mask | number |
| Mortar | number |

> Use it as a mask: into **Factor** of Mix colour, **Position** of a Color ramp, **Roughness / Metallic / Alpha / Height** of the output, or **A/B** of Math.

Example: Texture coordinate (World) → Mapping (scale 4) → **Bricks** (input Coords) → Color ramp → Output > Base color.

## Dots

A grid of round dots.

*Works in: material, object shader, camera effect*

**Inputs**

| Input | Type | When not connected |
|---|---|---|
| Coords | vector | the object's UV |

**Parameters** (on the node or in the Inspector)

| Parameter | Default | Range | Choices |
|---|---|---|---|
| scale | 10 | 1..200 | - |
| size | 0.3 | 0.01..0.5 | - |

**Outputs**

| Output | Type |
|---|---|
| Factor | number |
| Inverted | number |
| Sharp | number |

> Use it as a mask: into **Factor** of Mix colour, **Position** of a Color ramp, **Roughness / Metallic / Alpha / Height** of the output, or **A/B** of Math.

Example: Texture coordinate (World) → Mapping (scale 4) → **Dots** (input Coords) → Color ramp → Output > Base color.

## Caustics

Moving light patterns like under water.

*Works in: material, object shader*

**Inputs**

| Input | Type | When not connected |
|---|---|---|
| Coords | vector | world position |

**Parameters** (on the node or in the Inspector)

| Parameter | Default | Range | Choices |
|---|---|---|---|
| scale | 1.5 | 0.05..50 | - |
| speed | 0.6 | 0..10 | - |

**Output**: number

> Use it as a mask: into **Factor** of Mix colour, **Position** of a Color ramp, **Roughness / Metallic / Alpha / Height** of the output, or **A/B** of Math.

Example: Texture coordinate (World) → Mapping (scale 4) → **Caustics** (input Coords) → Color ramp → Output > Base color.

## Hexagons

Honeycomb pattern edges.

*Works in: material, object shader, camera effect*

**Inputs**

| Input | Type | When not connected |
|---|---|---|
| Coords | vector | the object's UV |

**Parameters** (on the node or in the Inspector)

| Parameter | Default | Range | Choices |
|---|---|---|---|
| scale | 6 | 0.5..100 | - |
| edge width | 0.06 | 0..0.5 | - |

**Output**: number

> Use it as a mask: into **Factor** of Mix colour, **Position** of a Color ramp, **Roughness / Metallic / Alpha / Height** of the output, or **A/B** of Math.

Example: Texture coordinate (World) → Mapping (scale 4) → **Hexagons** (input Coords) → Color ramp → Output > Base color.

## Wave rings

Animated rings spreading from the centre.

*Works in: material, object shader, camera effect*

**Inputs**

| Input | Type | When not connected |
|---|---|---|
| Coords | vector | the object's UV |

**Parameters** (on the node or in the Inspector)

| Parameter | Default | Range | Choices |
|---|---|---|---|
| frequency | 20 | 1..200 | - |
| speed | 2 | -20..20 | - |

**Output**: number

> Use it as a mask: into **Factor** of Mix colour, **Position** of a Color ramp, **Roughness / Metallic / Alpha / Height** of the output, or **A/B** of Math.

Example: Texture coordinate (World) → Mapping (scale 4) → **Wave rings** (input Coords) → Color ramp → Output > Base color.

## Linear gradient

0..1 from left to right (X of the coordinates).

*Works in: material, object shader, camera effect*

**Inputs**

| Input | Type | When not connected |
|---|---|---|
| Coords | vector | the object's UV |

**Output**: number

> Use it as a mask: into **Factor** of Mix colour, **Position** of a Color ramp, **Roughness / Metallic / Alpha / Height** of the output, or **A/B** of Math.

Example: Texture coordinate (World) → Mapping (scale 4) → **Linear gradient** (input Coords) → Color ramp → Output > Base color.

## Radial gradient

1 in the centre, 0 at the edge.

*Works in: material, object shader, camera effect*

**Inputs**

| Input | Type | When not connected |
|---|---|---|
| Coords | vector | the object's UV |

**Output**: number

> Use it as a mask: into **Factor** of Mix colour, **Position** of a Color ramp, **Roughness / Metallic / Alpha / Height** of the output, or **A/B** of Math.

Example: Texture coordinate (World) → Mapping (scale 4) → **Radial gradient** (input Coords) → Color ramp → Output > Base color.

## Angular gradient

0..1 around the centre like a clock hand.

*Works in: material, object shader, camera effect*

**Inputs**

| Input | Type | When not connected |
|---|---|---|
| Coords | vector | the object's UV |

**Output**: number

> Use it as a mask: into **Factor** of Mix colour, **Position** of a Color ramp, **Roughness / Metallic / Alpha / Height** of the output, or **A/B** of Math.

Example: Texture coordinate (World) → Mapping (scale 4) → **Angular gradient** (input Coords) → Color ramp → Output > Base color.

## Circle

A soft disc.

*Works in: material, object shader, camera effect*

**Inputs**

| Input | Type | When not connected |
|---|---|---|
| Coords | vector | the object's UV |

**Parameters** (on the node or in the Inspector)

| Parameter | Default | Range | Choices |
|---|---|---|---|
| radius | 0.3 | 0..1 | - |
| softness | 0.02 | 0..0.5 | - |

**Output**: number

> Use it as a mask: into **Factor** of Mix colour, **Position** of a Color ramp, **Roughness / Metallic / Alpha / Height** of the output, or **A/B** of Math.

Example: Texture coordinate (World) → Mapping (scale 4) → **Circle** (input Coords) → Color ramp → Output > Base color.

## Box

A soft rectangle.

*Works in: material, object shader, camera effect*

**Inputs**

| Input | Type | When not connected |
|---|---|---|
| Coords | vector | the object's UV |

**Parameters** (on the node or in the Inspector)

| Parameter | Default | Range | Choices |
|---|---|---|---|
| width | 0.6 | 0..1 | - |
| height | 0.4 | 0..1 | - |
| softness | 0.02 | 0..0.5 | - |

**Output**: number

> Use it as a mask: into **Factor** of Mix colour, **Position** of a Color ramp, **Roughness / Metallic / Alpha / Height** of the output, or **A/B** of Math.

Example: Texture coordinate (World) → Mapping (scale 4) → **Box** (input Coords) → Color ramp → Output > Base color.

## Wood rings

Tree-ring pattern for wood.

*Works in: material, object shader*

**Inputs**

| Input | Type | When not connected |
|---|---|---|
| Coords | vector | world position |

**Parameters** (on the node or in the Inspector)

| Parameter | Default | Range | Choices |
|---|---|---|---|
| scale | 4 | 0.1..50 | - |
| wobble | 0.3 | 0..2 | - |

**Outputs**

| Output | Type |
|---|---|
| Factor | number |
| Inverted | number |
| Sharp | number |

> Use it as a mask: into **Factor** of Mix colour, **Position** of a Color ramp, **Roughness / Metallic / Alpha / Height** of the output, or **A/B** of Math.

Example: Texture coordinate (World) → Mapping (scale 4) → **Wood rings** (input Coords) → Color ramp → Output > Base color.

## Marble veins

Wavy veins for marble.

*Works in: material, object shader*

**Inputs**

| Input | Type | When not connected |
|---|---|---|
| Coords | vector | world position |

**Parameters** (on the node or in the Inspector)

| Parameter | Default | Range | Choices |
|---|---|---|---|
| scale | 1.5 | 0.05..20 | - |
| warp | 4 | 0..20 | - |

**Outputs**

| Output | Type |
|---|---|
| Factor | number |
| Inverted | number |
| Sharp | number |

> Use it as a mask: into **Factor** of Mix colour, **Position** of a Color ramp, **Roughness / Metallic / Alpha / Height** of the output, or **A/B** of Math.

Example: Texture coordinate (World) → Mapping (scale 4) → **Marble veins** (input Coords) → Color ramp → Output > Base color.

## Grid lines

A grid of thin lines (white lines on black).

*Works in: material, object shader, camera effect*

**Inputs**

| Input | Type | When not connected |
|---|---|---|
| Coords | vector | the object's UV |

**Parameters** (on the node or in the Inspector)

| Parameter | Default | Range | Choices |
|---|---|---|---|
| cells | 8 | 1..200 | - |
| line width | 0.05 | 0.001..0.5 | - |

**Output**: number

> Use it as a mask: into **Factor** of Mix colour, **Position** of a Color ramp, **Roughness / Metallic / Alpha / Height** of the output, or **A/B** of Math.

Example: Texture coordinate (World) → Mapping (scale 4) → **Grid lines** (input Coords) → Color ramp → Output > Base color.

## Planks

Wooden floor boards with random tones and gaps.

*Works in: material, object shader, camera effect*

**Inputs**

| Input | Type | When not connected |
|---|---|---|
| Coords | vector | the object's UV |

**Parameters** (on the node or in the Inspector)

| Parameter | Default | Range | Choices |
|---|---|---|---|
| rows | 6 | 1..100 | - |
| plank length | 3 | 0.2..40 | - |
| gap | 0.03 | 0..0.3 | - |

**Output**: number

> Use it as a mask: into **Factor** of Mix colour, **Position** of a Color ramp, **Roughness / Metallic / Alpha / Height** of the output, or **A/B** of Math.

Example: Texture coordinate (World) → Mapping (scale 4) → **Planks** (input Coords) → Color ramp → Output > Base color.

## Weave

Over-under fabric weave pattern.

*Works in: material, object shader, camera effect*

**Inputs**

| Input | Type | When not connected |
|---|---|---|
| Coords | vector | the object's UV |

**Parameters** (on the node or in the Inspector)

| Parameter | Default | Range | Choices |
|---|---|---|---|
| threads | 12 | 1..200 | - |

**Output**: number

> Use it as a mask: into **Factor** of Mix colour, **Position** of a Color ramp, **Roughness / Metallic / Alpha / Height** of the output, or **A/B** of Math.

Example: Texture coordinate (World) → Mapping (scale 4) → **Weave** (input Coords) → Color ramp → Output > Base color.

## Wave texture

Bands or rings with sine, saw or triangle profile, bent by noise.

*Works in: material, object shader, camera effect*

**Inputs**

| Input | Type | When not connected |
|---|---|---|
| Vector | vector | world position |
| Scale | number | `3.0` |
| Distortion | number | `2.0` |
| Detail | number | `3.0` |

**Parameters** (on the node or in the Inspector)

| Parameter | Default | Range | Choices |
|---|---|---|---|
| type | 0 | 0..1 | 0 Bands, 1 Rings |
| profile | 0 | 0..2 | 0 Sine, 1 Saw, 2 Triangle |

**Outputs**

| Output | Type |
|---|---|
| Factor | number |
| Inverted | number |
| Sharp | number |

> Use it as a mask: into **Factor** of Mix colour, **Position** of a Color ramp, **Roughness / Metallic / Alpha / Height** of the output, or **A/B** of Math.

Example: Texture coordinate (World) → Mapping (scale 4) → **Wave texture** (input Vector) → Color ramp → Output > Base color.

## Gradient texture

Linear, quadratic, eased, diagonal, radial or spherical gradient.

*Works in: material, object shader, camera effect*

**Inputs**

| Input | Type | When not connected |
|---|---|---|
| Vector | vector | the object's UV |

**Parameters** (on the node or in the Inspector)

| Parameter | Default | Range | Choices |
|---|---|---|---|
| type | 0 | 0..6 | 0 Linear, 1 Quadratic, 2 Eased, 3 Diagonal, 4 Radial, 5 Spherical, 6 Quadratic sphere |

**Outputs**

| Output | Type |
|---|---|
| Factor | number |
| Inverted | number |
| Sharp | number |

> Use it as a mask: into **Factor** of Mix colour, **Position** of a Color ramp, **Roughness / Metallic / Alpha / Height** of the output, or **A/B** of Math.

Example: Texture coordinate (World) → Mapping (scale 4) → **Gradient texture** (input Vector) → Color ramp → Output > Base color.

## Brick texture

Bricks with two colours, mortar, row offset and a value per brick.

*Works in: material, object shader, camera effect*

**Inputs**

| Input | Type | When not connected |
|---|---|---|
| Vector | vector | the object's UV |
| Colour 1 | colour | `0.62, 0.24, 0.16` |
| Colour 2 | colour | `0.45, 0.17, 0.12` |
| Mortar | colour | `0.75, 0.72, 0.66` |
| Scale | number | `6.0` |

**Parameters** (on the node or in the Inspector)

| Parameter | Default | Range | Choices |
|---|---|---|---|
| brick ratio | 2 | 0.2..6 | - |
| row offset | 0.5 | 0..1 | - |
| mortar size | 0.05 | 0..0.3 | - |
| mortar smooth | 0.02 | 0..0.3 | - |
| colour variation | 0.6 | 0..1 | - |

**Outputs**

| Output | Type |
|---|---|
| Colour | colour |
| Mask | number |
| Brick value | number |
| Edge distance | number |

> Usually goes to **Output > Base color** or **Emission**, or to **A/B** of Mix colour. Into a number input it is converted to luminance.

Example: Texture coordinate (World) → Mapping (scale 4) → **Brick texture** (input Vector) → Color ramp → Output > Base color.

