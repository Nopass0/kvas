---
outline: [2, 3]
---

# Normals

Normal maps, bump from height, normal blending.

Generated from the node library. Socket types: **number** (grey circle), **colour** (yellow circle), **vector** (purple diamond). Any output can feed any input; when the types differ, pick a conversion on the input (right-click it). See [how nodes connect](../wiring).

## Normal map

Turns a normal-map colour into a surface direction. Connect the result to Output > Normal.

*Works in: material, object shader*

**Inputs**

| Input | Type | When not connected |
|---|---|---|
| Map | colour | `0.5, 0.5, 1.0` |
| UV | vector | the object's UV |

**Parameters** (on the node or in the Inspector)

| Parameter | Default | Range | Choices |
|---|---|---|---|
| strength | 1 | 0..4 | - |

**Output**: number

> Use it as a mask: into **Factor** of Mix colour, **Position** of a Color ramp, **Roughness / Metallic / Alpha / Height** of the output, or **A/B** of Math.

Example: Image texture (Colour) → **Normal map** (input Map) → Output > Base color.

## Bump from height

Makes bumps from a height value. Connect the result to Output > Normal.

*Works in: material, object shader*

**Inputs**

| Input | Type | When not connected |
|---|---|---|
| Height | number | `0.0` |

**Parameters** (on the node or in the Inspector)

| Parameter | Default | Range | Choices |
|---|---|---|---|
| strength | 1 | 0..10 | - |

**Output**: number

> Use it as a mask: into **Factor** of Mix colour, **Position** of a Color ramp, **Roughness / Metallic / Alpha / Height** of the output, or **A/B** of Math.

Example: Noise texture (Factor) → **Bump from height** (input Height) → Mix colour (Factor).

## Normal strength

Blends between the surface normal and the connected normal.

*Works in: material, object shader*

**Inputs**

| Input | Type | When not connected |
|---|---|---|
| Normal | vector | surface normal |

**Parameters** (on the node or in the Inspector)

| Parameter | Default | Range | Choices |
|---|---|---|---|
| strength | 1 | 0..4 | - |

**Output**: number

> Use it as a mask: into **Factor** of Mix colour, **Position** of a Color ramp, **Roughness / Metallic / Alpha / Height** of the output, or **A/B** of Math.

Example: Texture coordinate (World) → Mapping (scale 4) → **Normal strength** (input Normal) → Color ramp → Output > Base color.

## Normal blend

Adds a detail normal on top of a base normal.

*Works in: material, object shader*

**Inputs**

| Input | Type | When not connected |
|---|---|---|
| Base | vector | surface normal |
| Detail | vector | surface normal |

**Output**: number

> Use it as a mask: into **Factor** of Mix colour, **Position** of a Color ramp, **Roughness / Metallic / Alpha / Height** of the output, or **A/B** of Math.

Example: Texture coordinate (World) → Mapping (scale 4) → **Normal blend** (input Base) → Color ramp → Output > Base color.

## Normal to colour

Shows a normal as a colour (for previews and debugging).

*Works in: material, object shader*

**Inputs**

| Input | Type | When not connected |
|---|---|---|
| Normal | vector | surface normal |

**Output**: number

> Use it as a mask: into **Factor** of Mix colour, **Position** of a Color ramp, **Roughness / Metallic / Alpha / Height** of the output, or **A/B** of Math.

Example: Texture coordinate (World) → Mapping (scale 4) → **Normal to colour** (input Normal) → Color ramp → Output > Base color.

## Noise bump

Bumps the surface with fractal noise. Connect to Output > Normal.

*Works in: material, object shader*

**Inputs**

| Input | Type | When not connected |
|---|---|---|
| Coords | vector | world position |

**Parameters** (on the node or in the Inspector)

| Parameter | Default | Range | Choices |
|---|---|---|---|
| scale | 4 | 0.05..100 | - |
| strength | 1 | 0..10 | - |

**Output**: number

> Use it as a mask: into **Factor** of Mix colour, **Position** of a Color ramp, **Roughness / Metallic / Alpha / Height** of the output, or **A/B** of Math.

Example: Texture coordinate (World) → Mapping (scale 4) → **Noise bump** (input Coords) → Color ramp → Output > Base color.

## Height to normal

Makes a normal-map colour from the height in a texture slot. Feed it to Normal map.

*Works in: material, object shader*

**Inputs**

| Input | Type | When not connected |
|---|---|---|
| UV | vector | the object's UV |

**Parameters** (on the node or in the Inspector)

| Parameter | Default | Range | Choices |
|---|---|---|---|
| slot | 0 | 0..2 | - |
| tiling | 1 | 0.01..100 | - |
| strength | 4 | 0..32 | - |

**Output**: number

> Use it as a mask: into **Factor** of Mix colour, **Position** of a Color ramp, **Roughness / Metallic / Alpha / Height** of the output, or **A/B** of Math.

Example: Texture coordinate (World) → Mapping (scale 4) → **Height to normal** (input UV) → Color ramp → Output > Base color.

## Bump

Surface relief from a height value; connect to Output > Normal.

*Works in: material, object shader*

**Inputs**

| Input | Type | When not connected |
|---|---|---|
| Height | number | `0.0` |
| Strength | number | `1.0` |
| Distance | number | `0.1` |

**Parameters** (on the node or in the Inspector)

| Parameter | Default | Range | Choices |
|---|---|---|---|
| invert | 0 | 0..1 | - |

**Output**: number

> Use it as a mask: into **Factor** of Mix colour, **Position** of a Color ramp, **Roughness / Metallic / Alpha / Height** of the output, or **A/B** of Math.

Example: Noise texture (Factor) → **Bump** (input Height) → Mix colour (Factor).

