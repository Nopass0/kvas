---
outline: [2, 3]
---

# Noise

Fractal, Perlin, Voronoi, cracks, clouds, grunge. They show a preview by default.

Generated from the node library. Socket types: **number** (grey circle), **colour** (yellow circle), **vector** (purple diamond). Any output can feed any input; when the types differ, pick a conversion on the input (right-click it). See [how nodes connect](../wiring).

## Noise

Simple smooth value noise (2D, uses the X and Z of the coordinates).

*Works in: material, object shader, camera effect*

**Inputs**

| Input | Type | When not connected |
|---|---|---|
| Coords | vector | world position |

**Parameters** (on the node or in the Inspector)

| Parameter | Default | Range | Choices |
|---|---|---|---|
| scale | 2 | 0.01..100 | - |

**Outputs**

| Output | Type |
|---|---|
| Factor | number |
| Inverted | number |
| Sharp | number |

> Use it as a mask: into **Factor** of Mix colour, **Position** of a Color ramp, **Roughness / Metallic / Alpha / Height** of the output, or **A/B** of Math.

Example: Texture coordinate (World) → Mapping (scale 4) → **Noise** (input Coords) → Color ramp → Output > Base color.

## Voronoi

Cell pattern: distance to the nearest random point.

*Works in: material, object shader, camera effect*

**Inputs**

| Input | Type | When not connected |
|---|---|---|
| Coords | vector | world position |

**Parameters** (on the node or in the Inspector)

| Parameter | Default | Range | Choices |
|---|---|---|---|
| scale | 3 | 0.01..100 | - |

**Outputs**

| Output | Type |
|---|---|
| Factor | number |
| Inverted | number |
| Sharp | number |

> Use it as a mask: into **Factor** of Mix colour, **Position** of a Color ramp, **Roughness / Metallic / Alpha / Height** of the output, or **A/B** of Math.

Example: Texture coordinate (World) → Mapping (scale 4) → **Voronoi** (input Coords) → Color ramp → Output > Base color.

## Cell colour

A random colour per Voronoi cell.

*Works in: material, object shader, camera effect*

**Inputs**

| Input | Type | When not connected |
|---|---|---|
| Coords | vector | world position |

**Parameters** (on the node or in the Inspector)

| Parameter | Default | Range | Choices |
|---|---|---|---|
| scale | 3 | 0.01..100 | - |

**Outputs**

| Output | Type |
|---|---|
| Colour | colour |
| R | number |
| G | number |
| B | number |
| Luma | number |

> Usually goes to **Output > Base color** or **Emission**, or to **A/B** of Mix colour. Into a number input it is converted to luminance.

Example: Texture coordinate (World) → Mapping (scale 4) → **Cell colour** (input Coords) → Color ramp → Output > Base color.

## Noise 3D

Smooth 3D noise that does not stretch on any surface.

*Works in: material, object shader*

**Inputs**

| Input | Type | When not connected |
|---|---|---|
| Coords | vector | world position |

**Parameters** (on the node or in the Inspector)

| Parameter | Default | Range | Choices |
|---|---|---|---|
| scale | 2 | 0.01..100 | - |

**Output**: number

> Use it as a mask: into **Factor** of Mix colour, **Position** of a Color ramp, **Roughness / Metallic / Alpha / Height** of the output, or **A/B** of Math.

Example: Texture coordinate (World) → Mapping (scale 4) → **Noise 3D** (input Coords) → Color ramp → Output > Base color.

## Ridged noise

Noise with sharp ridges (mountains, veins).

*Works in: material, object shader*

**Inputs**

| Input | Type | When not connected |
|---|---|---|
| Coords | vector | world position |

**Parameters** (on the node or in the Inspector)

| Parameter | Default | Range | Choices |
|---|---|---|---|
| scale | 2 | 0.01..100 | - |

**Output**: number

> Use it as a mask: into **Factor** of Mix colour, **Position** of a Color ramp, **Roughness / Metallic / Alpha / Height** of the output, or **A/B** of Math.

Example: Texture coordinate (World) → Mapping (scale 4) → **Ridged noise** (input Coords) → Color ramp → Output > Base color.

## Turbulence

Billowy noise with creases (smoke, fire).

*Works in: material, object shader*

**Inputs**

| Input | Type | When not connected |
|---|---|---|
| Coords | vector | world position |

**Parameters** (on the node or in the Inspector)

| Parameter | Default | Range | Choices |
|---|---|---|---|
| scale | 2 | 0.01..100 | - |

**Output**: number

> Use it as a mask: into **Factor** of Mix colour, **Position** of a Color ramp, **Roughness / Metallic / Alpha / Height** of the output, or **A/B** of Math.

Example: Texture coordinate (World) → Mapping (scale 4) → **Turbulence** (input Coords) → Color ramp → Output > Base color.

## White noise

A random grey value per cell, no smoothing (TV static).

*Works in: material, object shader, camera effect*

**Inputs**

| Input | Type | When not connected |
|---|---|---|
| Coords | vector | the object's UV |

**Parameters** (on the node or in the Inspector)

| Parameter | Default | Range | Choices |
|---|---|---|---|
| cells | 64 | 1..1000 | - |

**Output**: number

> Use it as a mask: into **Factor** of Mix colour, **Position** of a Color ramp, **Roughness / Metallic / Alpha / Height** of the output, or **A/B** of Math.

Example: Texture coordinate (World) → Mapping (scale 4) → **White noise** (input Coords) → Color ramp → Output > Base color.

## Sparkle

Rare bright dots (stars, glitter).

*Works in: material, object shader, camera effect*

**Inputs**

| Input | Type | When not connected |
|---|---|---|
| Coords | vector | the object's UV |

**Parameters** (on the node or in the Inspector)

| Parameter | Default | Range | Choices |
|---|---|---|---|
| cells | 80 | 4..1000 | - |
| density | 0.03 | 0..1 | - |

**Output**: number

> Use it as a mask: into **Factor** of Mix colour, **Position** of a Color ramp, **Roughness / Metallic / Alpha / Height** of the output, or **A/B** of Math.

Example: Texture coordinate (World) → Mapping (scale 4) → **Sparkle** (input Coords) → Color ramp → Output > Base color.

## Fractal noise

Layered noise with control over octaves, lacunarity and gain. Types: fBm, ridged, billow.

*Works in: material, object shader, camera effect*

**Inputs**

| Input | Type | When not connected |
|---|---|---|
| Coords | vector | world position |

**Parameters** (on the node or in the Inspector)

| Parameter | Default | Range | Choices |
|---|---|---|---|
| scale | 3 | 0.01..100 | - |
| octaves | 5 | 1..8 | - |
| lacunarity | 2 | 1.2..4 | - |
| gain | 0.5 | 0.1..0.9 | - |
| type | 0 | 0..2 | - |

**Outputs**

| Output | Type |
|---|---|
| Factor | number |
| Inverted | number |
| Sharp | number |

> Use it as a mask: into **Factor** of Mix colour, **Position** of a Color ramp, **Roughness / Metallic / Alpha / Height** of the output, or **A/B** of Math.

Example: Texture coordinate (World) → Mapping (scale 4) → **Fractal noise** (input Coords) → Color ramp → Output > Base color.

## Perlin noise

Smooth gradient noise (0..1).

*Works in: material, object shader, camera effect*

**Inputs**

| Input | Type | When not connected |
|---|---|---|
| Coords | vector | world position |

**Parameters** (on the node or in the Inspector)

| Parameter | Default | Range | Choices |
|---|---|---|---|
| scale | 4 | 0.01..100 | - |

**Outputs**

| Output | Type |
|---|---|
| Factor | number |
| Inverted | number |
| Sharp | number |

> Use it as a mask: into **Factor** of Mix colour, **Position** of a Color ramp, **Roughness / Metallic / Alpha / Height** of the output, or **A/B** of Math.

Example: Texture coordinate (World) → Mapping (scale 4) → **Perlin noise** (input Coords) → Color ramp → Output > Base color.

## Voronoi cells

3D cell noise. Outputs: distance, cell edges, random id per cell, second distance.

*Works in: material, object shader, camera effect*

**Inputs**

| Input | Type | When not connected |
|---|---|---|
| Coords | vector | world position |

**Parameters** (on the node or in the Inspector)

| Parameter | Default | Range | Choices |
|---|---|---|---|
| scale | 4 | 0.01..100 | - |
| randomness | 1 | 0..1 | - |
| output | 0 | 0..3 | - |

**Output**: number

> Use it as a mask: into **Factor** of Mix colour, **Position** of a Color ramp, **Roughness / Metallic / Alpha / Height** of the output, or **A/B** of Math.

Example: Texture coordinate (World) → Mapping (scale 4) → **Voronoi cells** (input Coords) → Color ramp → Output > Base color.

## Cracks

Thin cracks along cell borders (dry ground, ice, shattered stone).

*Works in: material, object shader, camera effect*

**Inputs**

| Input | Type | When not connected |
|---|---|---|
| Coords | vector | world position |

**Parameters** (on the node or in the Inspector)

| Parameter | Default | Range | Choices |
|---|---|---|---|
| scale | 3 | 0.01..100 | - |
| width | 0.08 | 0.005..0.5 | - |

**Outputs**

| Output | Type |
|---|---|
| Factor | number |
| Inverted | number |
| Sharp | number |

> Use it as a mask: into **Factor** of Mix colour, **Position** of a Color ramp, **Roughness / Metallic / Alpha / Height** of the output, or **A/B** of Math.

Example: Texture coordinate (World) → Mapping (scale 4) → **Cracks** (input Coords) → Color ramp → Output > Base color.

## Warped noise

Noise bent by other noise - swirly, organic shapes.

*Works in: material, object shader, camera effect*

**Inputs**

| Input | Type | When not connected |
|---|---|---|
| Coords | vector | world position |

**Parameters** (on the node or in the Inspector)

| Parameter | Default | Range | Choices |
|---|---|---|---|
| scale | 3 | 0.01..100 | - |
| warp | 1.5 | 0..8 | - |

**Output**: number

> Use it as a mask: into **Factor** of Mix colour, **Position** of a Color ramp, **Roughness / Metallic / Alpha / Height** of the output, or **A/B** of Math.

Example: Texture coordinate (World) → Mapping (scale 4) → **Warped noise** (input Coords) → Color ramp → Output > Base color.

## Clouds

Soft cloud-like blobs. Coverage sets how much is filled.

*Works in: material, object shader, camera effect*

**Inputs**

| Input | Type | When not connected |
|---|---|---|
| Coords | vector | world position |

**Parameters** (on the node or in the Inspector)

| Parameter | Default | Range | Choices |
|---|---|---|---|
| scale | 2 | 0.01..100 | - |
| coverage | 0.45 | 0..1 | - |
| softness | 0.3 | 0.01..1 | - |

**Outputs**

| Output | Type |
|---|---|
| Factor | number |
| Inverted | number |
| Sharp | number |

> Use it as a mask: into **Factor** of Mix colour, **Position** of a Color ramp, **Roughness / Metallic / Alpha / Height** of the output, or **A/B** of Math.

Example: Texture coordinate (World) → Mapping (scale 4) → **Clouds** (input Coords) → Color ramp → Output > Base color.

## Grunge

Dirt and wear mask. Multiply with a base colour or use for roughness.

*Works in: material, object shader, camera effect*

**Inputs**

| Input | Type | When not connected |
|---|---|---|
| Coords | vector | world position |

**Parameters** (on the node or in the Inspector)

| Parameter | Default | Range | Choices |
|---|---|---|---|
| scale | 3 | 0.01..100 | - |
| contrast | 1.8 | 0.1..8 | - |

**Outputs**

| Output | Type |
|---|---|
| Factor | number |
| Inverted | number |
| Sharp | number |

> Use it as a mask: into **Factor** of Mix colour, **Position** of a Color ramp, **Roughness / Metallic / Alpha / Height** of the output, or **A/B** of Math.

Example: Texture coordinate (World) → Mapping (scale 4) → **Grunge** (input Coords) → Color ramp → Output > Base color.

## Scratches

Thin stretched scratches at an angle.

*Works in: material, object shader, camera effect*

**Inputs**

| Input | Type | When not connected |
|---|---|---|
| Coords | vector | the object's UV |

**Parameters** (on the node or in the Inspector)

| Parameter | Default | Range | Choices |
|---|---|---|---|
| density | 40 | 2..400 | - |
| angle | 0.4 | -3.15..3.15 | - |

**Output**: number

> Use it as a mask: into **Factor** of Mix colour, **Position** of a Color ramp, **Roughness / Metallic / Alpha / Height** of the output, or **A/B** of Math.

Example: Texture coordinate (World) → Mapping (scale 4) → **Scratches** (input Coords) → Color ramp → Output > Base color.

## Noise texture

Fractal noise with scale, detail, roughness and distortion inputs; outputs a factor and a colour.

*Works in: material, object shader, camera effect*

**Inputs**

| Input | Type | When not connected |
|---|---|---|
| Vector | vector | world position |
| Scale | number | `4.0` |
| Detail | number | `5.0` |
| Roughness | number | `0.5` |
| Distortion | number | `0.0` |

**Parameters** (on the node or in the Inspector)

| Parameter | Default | Range | Choices |
|---|---|---|---|
| type | 0 | 0..2 | 0 fBm, 1 Ridged, 2 Billow |
| lacunarity | 2 | 1.2..4 | - |

**Outputs**

| Output | Type |
|---|---|
| Factor | number |
| Colour | colour |

> Use it as a mask: into **Factor** of Mix colour, **Position** of a Color ramp, **Roughness / Metallic / Alpha / Height** of the output, or **A/B** of Math.

Example: Texture coordinate (World) → Mapping (scale 4) → **Noise texture** (input Vector) → Color ramp → Output > Base color.

## Voronoi texture

Cells: distance, edges, a value and a colour per cell.

*Works in: material, object shader, camera effect*

**Inputs**

| Input | Type | When not connected |
|---|---|---|
| Vector | vector | world position |
| Scale | number | `5.0` |
| Randomness | number | `1.0` |

**Parameters** (on the node or in the Inspector)

| Parameter | Default | Range | Choices |
|---|---|---|---|
| feature | 0 | 0..3 | 0 Distance (F1), 1 Second (F2), 2 Edge (F2-F1), 3 Cell value |

**Outputs**

| Output | Type |
|---|---|
| Factor | number |
| Distance | number |
| Edge | number |
| Cell value | number |
| Cell colour | colour |

> Use it as a mask: into **Factor** of Mix colour, **Position** of a Color ramp, **Roughness / Metallic / Alpha / Height** of the output, or **A/B** of Math.

Example: Texture coordinate (World) → Mapping (scale 4) → **Voronoi texture** (input Vector) → Color ramp → Output > Base color.

