---
outline: [2, 3]
---

# Input

Constants, textures, time, UV.

Generated from the node library. Socket types: **number** (grey circle), **colour** (yellow circle), **vector** (purple diamond). Any output can feed any input; when the types differ, pick a conversion on the input (right-click it). See [how nodes connect](../wiring).

## Color

A constant colour.

*Works in: material, object shader, camera effect*

**Parameters** (on the node or in the Inspector)

| Parameter | Default | Range | Choices |
|---|---|---|---|
| colour | 0.80, 0.80, 0.80 | 0..1 | - |

**Outputs**

| Output | Type |
|---|---|
| Colour | colour |
| R | number |
| G | number |
| B | number |
| Luma | number |

> Usually goes to **Output > Base color** or **Emission**, or to **A/B** of Mix colour. Into a number input it is converted to luminance.

## Value

A constant number.

*Works in: material, object shader, camera effect*

**Parameters** (on the node or in the Inspector)

| Parameter | Default | Range | Choices |
|---|---|---|---|
| value | 0.5 | -100..100 | - |

**Output**: number

> Use it as a mask: into **Factor** of Mix colour, **Position** of a Color ramp, **Roughness / Metallic / Alpha / Height** of the output, or **A/B** of Math.

## Texture

Samples one of the material images (slot 0 albedo, 1 normal, 2 ORM).

*Works in: material, object shader*

**Inputs**

| Input | Type | When not connected |
|---|---|---|
| UV | vector | the object's UV |

**Parameters** (on the node or in the Inspector)

| Parameter | Default | Range | Choices |
|---|---|---|---|
| slot | 0 | 0..2 | 0 Slot 0 (albedo), 1 Slot 1 (normal), 2 Slot 2 (ORM) |
| tiling | 1 | 0.01..100 | - |

**Outputs**

| Output | Type |
|---|---|
| Colour | colour |
| R | number |
| G | number |
| B | number |
| Luma | number |

> Usually goes to **Output > Base color** or **Emission**, or to **A/B** of Mix colour. Into a number input it is converted to luminance.

Example: Texture coordinate (World) → Mapping (scale 4) → **Texture** (input UV) → Color ramp → Output > Base color.

## Time

Seconds since the game started.

*Works in: material, object shader, camera effect*

**Outputs**

| Output | Type |
|---|---|
| Seconds | number |
| Sine | number |
| Fraction | number |

> Use it as a mask: into **Factor** of Mix colour, **Position** of a Color ramp, **Roughness / Metallic / Alpha / Height** of the output, or **A/B** of Math.

## Object colour

The colour set in the MeshRenderer of the object.

*Works in: material, object shader*

**Output**: colour

> Usually goes to **Output > Base color** or **Emission**, or to **A/B** of Mix colour. Into a number input it is converted to luminance.

## Triplanar texture

Projects a material image from three world axes, no stretching on any slope.

*Works in: material, object shader*

**Parameters** (on the node or in the Inspector)

| Parameter | Default | Range | Choices |
|---|---|---|---|
| slot | 0 | 0..2 | 0 Slot 0 (albedo), 1 Slot 1 (normal), 2 Slot 2 (ORM) |
| scale | 1 | 0.01..50 | - |
| sharpness | 4 | 1..32 | - |

**Outputs**

| Output | Type |
|---|---|
| Colour | colour |
| R | number |
| G | number |
| B | number |
| Luma | number |

> Usually goes to **Output > Base color** or **Emission**, or to **A/B** of Mix colour. Into a number input it is converted to luminance.

## Image texture

An image file of the project. Pick it in the node; up to three images per material (slots 0..2). Outputs colour, channels and brightness.

*Works in: material, object shader*

**Inputs**

| Input | Type | When not connected |
|---|---|---|
| Vector | vector | the object's UV |

**Parameters** (on the node or in the Inspector)

| Parameter | Default | Range | Choices |
|---|---|---|---|
| slot | 0 | 0..2 | 0 Slot 0 (albedo), 1 Slot 1 (normal), 2 Slot 2 (ORM) |
| tiling x | 1 | 0.01..100 | - |
| tiling y | 1 | 0.01..100 | - |
| offset x | 0 | -10..10 | - |
| offset y | 0 | -10..10 | - |
| rotation | 0 | -1..1 | - |

**Outputs**

| Output | Type |
|---|---|
| Colour | colour |
| R | number |
| G | number |
| B | number |
| Luma | number |

> Usually goes to **Output > Base color** or **Emission**, or to **A/B** of Mix colour. Into a number input it is converted to luminance.

Example: Texture coordinate (World) → Mapping (scale 4) → **Image texture** (input Vector) → Color ramp → Output > Base color.

## RGB

A colour with R, G, B and brightness outputs.

*Works in: material, object shader, camera effect*

**Inputs**

| Input | Type | When not connected |
|---|---|---|
| Colour | colour | `0.8, 0.5, 0.2` |

**Outputs**

| Output | Type |
|---|---|
| Colour | colour |
| R | number |
| G | number |
| B | number |
| Luma | number |

> Usually goes to **Output > Base color** or **Emission**, or to **A/B** of Mix colour. Into a number input it is converted to luminance.

Example: Image texture (Colour) → **RGB** (input Colour) → Output > Base color.

