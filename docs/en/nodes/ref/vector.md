---
outline: [2, 3]
---

# Vectors

Components, dot and cross products, rotations, coordinates.

Generated from the node library. Socket types: **number** (grey circle), **colour** (yellow circle), **vector** (purple diamond). Any output can feed any input; when the types differ, pick a conversion on the input (right-click it). See [how nodes connect](../wiring).

## Red

Copies the red channel to all channels.

*Works in: material, object shader, camera effect*

**Inputs**

| Input | Type | When not connected |
|---|---|---|
| Vector | vector | `0.0` |

**Output**: vector

> Feeds **Vector / Coords / UV** inputs of textures and noises, or **Normal** of the output. For one channel pick the X/Y/Z conversion.

Example: Texture coordinate (World) → Mapping (scale 4) → **Red** (input Vector) → Color ramp → Output > Base color.

## Green

Copies the green channel to all channels.

*Works in: material, object shader, camera effect*

**Inputs**

| Input | Type | When not connected |
|---|---|---|
| Vector | vector | `0.0` |

**Output**: vector

> Feeds **Vector / Coords / UV** inputs of textures and noises, or **Normal** of the output. For one channel pick the X/Y/Z conversion.

Example: Texture coordinate (World) → Mapping (scale 4) → **Green** (input Vector) → Color ramp → Output > Base color.

## Blue

Copies the blue channel to all channels.

*Works in: material, object shader, camera effect*

**Inputs**

| Input | Type | When not connected |
|---|---|---|
| Vector | vector | `0.0` |

**Output**: vector

> Feeds **Vector / Coords / UV** inputs of textures and noises, or **Normal** of the output. For one channel pick the X/Y/Z conversion.

Example: Texture coordinate (World) → Mapping (scale 4) → **Blue** (input Vector) → Color ramp → Output > Base color.

## Combine

Builds a vector from the red channels of three inputs.

*Works in: material, object shader, camera effect*

**Inputs**

| Input | Type | When not connected |
|---|---|---|
| R | number | `0.0` |
| G | number | `0.0` |
| B | number | `0.0` |

**Outputs**

| Output | Type |
|---|---|
| Vector | vector |
| Length | number |
| Normalized | vector |

> Feeds **Vector / Coords / UV** inputs of textures and noises, or **Normal** of the output. For one channel pick the X/Y/Z conversion.

Example: Noise texture (Factor) → **Combine** (input R) → Mix colour (Factor).

## Length

Length of a vector.

*Works in: material, object shader, camera effect*

**Inputs**

| Input | Type | When not connected |
|---|---|---|
| Vector | vector | `0.0` |

**Output**: number

> Use it as a mask: into **Factor** of Mix colour, **Position** of a Color ramp, **Roughness / Metallic / Alpha / Height** of the output, or **A/B** of Math.

Example: Texture coordinate (World) → Mapping (scale 4) → **Length** (input Vector) → Color ramp → Output > Base color.

## Distance

Distance between two points.

*Works in: material, object shader, camera effect*

**Inputs**

| Input | Type | When not connected |
|---|---|---|
| A | number | `0.0` |
| B | number | `0.0` |

**Output**: number

> Use it as a mask: into **Factor** of Mix colour, **Position** of a Color ramp, **Roughness / Metallic / Alpha / Height** of the output, or **A/B** of Math.

Example: Noise texture (Factor) → **Distance** (input A) → Mix colour (Factor).

## Dot

Dot product (how much two directions agree).

*Works in: material, object shader, camera effect*

**Inputs**

| Input | Type | When not connected |
|---|---|---|
| A | number | `0.0` |
| B | number | `0.0` |

**Output**: number

> Use it as a mask: into **Factor** of Mix colour, **Position** of a Color ramp, **Roughness / Metallic / Alpha / Height** of the output, or **A/B** of Math.

Example: Noise texture (Factor) → **Dot** (input A) → Mix colour (Factor).

## Cross

Cross product (a direction perpendicular to both).

*Works in: material, object shader, camera effect*

**Inputs**

| Input | Type | When not connected |
|---|---|---|
| A | number | `0.0` |
| B | number | `0.0` |

**Output**: vector

> Feeds **Vector / Coords / UV** inputs of textures and noises, or **Normal** of the output. For one channel pick the X/Y/Z conversion.

Example: Noise texture (Factor) → **Cross** (input A) → Mix colour (Factor).

## Normalize

Scales a vector to length 1.

*Works in: material, object shader, camera effect*

**Inputs**

| Input | Type | When not connected |
|---|---|---|
| Vector | vector | `0.0, 1.0, 0.0` |

**Output**: vector

> Feeds **Vector / Coords / UV** inputs of textures and noises, or **Normal** of the output. For one channel pick the X/Y/Z conversion.

Example: Texture coordinate (World) → Mapping (scale 4) → **Normalize** (input Vector) → Color ramp → Output > Base color.

## Reflect

Mirrors a direction around a normal.

*Works in: material, object shader, camera effect*

**Inputs**

| Input | Type | When not connected |
|---|---|---|
| Incident | number | `0.0, -1.0, 0.0` |
| Normal | vector | `0.0, 1.0, 0.0` |

**Output**: vector

> Feeds **Vector / Coords / UV** inputs of textures and noises, or **Normal** of the output. For one channel pick the X/Y/Z conversion.

Example: Noise texture (Factor) → **Reflect** (input Incident) → Mix colour (Factor).

## Rotate coords

Rotates texture coordinates around the centre.

*Works in: material, object shader, camera effect*

**Inputs**

| Input | Type | When not connected |
|---|---|---|
| Coords | vector | the object's UV |

**Parameters** (on the node or in the Inspector)

| Parameter | Default | Range | Choices |
|---|---|---|---|
| angle | 0 | -6.3..6.3 | - |

**Output**: vector

> Feeds **Vector / Coords / UV** inputs of textures and noises, or **Normal** of the output. For one channel pick the X/Y/Z conversion.

Example: Texture coordinate (World) → Mapping (scale 4) → **Rotate coords** (input Coords) → Color ramp → Output > Base color.

## Scale offset coords

Tiles and shifts texture coordinates.

*Works in: material, object shader, camera effect*

**Inputs**

| Input | Type | When not connected |
|---|---|---|
| Coords | vector | the object's UV |

**Parameters** (on the node or in the Inspector)

| Parameter | Default | Range | Choices |
|---|---|---|---|
| scale x | 1 | 0.01..100 | - |
| scale y | 1 | 0.01..100 | - |
| offset x | 0 | -100..100 | - |
| offset y | 0 | -100..100 | - |

**Output**: vector

> Feeds **Vector / Coords / UV** inputs of textures and noises, or **Normal** of the output. For one channel pick the X/Y/Z conversion.

Example: Texture coordinate (World) → Mapping (scale 4) → **Scale offset coords** (input Coords) → Color ramp → Output > Base color.

## Scroll coords

Moves coordinates over time (flowing water, conveyor belts).

*Works in: material, object shader, camera effect*

**Inputs**

| Input | Type | When not connected |
|---|---|---|
| Coords | vector | the object's UV |

**Parameters** (on the node or in the Inspector)

| Parameter | Default | Range | Choices |
|---|---|---|---|
| speed x | 0.1 | -10..10 | - |
| speed y | 0 | -10..10 | - |

**Output**: vector

> Feeds **Vector / Coords / UV** inputs of textures and noises, or **Normal** of the output. For one channel pick the X/Y/Z conversion.

Example: Texture coordinate (World) → Mapping (scale 4) → **Scroll coords** (input Coords) → Color ramp → Output > Base color.

## Polar coords

Converts coordinates to angle and radius.

*Works in: material, object shader, camera effect*

**Inputs**

| Input | Type | When not connected |
|---|---|---|
| Coords | vector | the object's UV |

**Output**: vector

> Feeds **Vector / Coords / UV** inputs of textures and noises, or **Normal** of the output. For one channel pick the X/Y/Z conversion.

Example: Texture coordinate (World) → Mapping (scale 4) → **Polar coords** (input Coords) → Color ramp → Output > Base color.

## Mirror tile

Repeats coordinates mirrored, so the seams disappear.

*Works in: material, object shader, camera effect*

**Inputs**

| Input | Type | When not connected |
|---|---|---|
| Coords | vector | the object's UV |

**Parameters** (on the node or in the Inspector)

| Parameter | Default | Range | Choices |
|---|---|---|---|
| tiles | 2 | 0.1..50 | - |

**Output**: vector

> Feeds **Vector / Coords / UV** inputs of textures and noises, or **Normal** of the output. For one channel pick the X/Y/Z conversion.

Example: Texture coordinate (World) → Mapping (scale 4) → **Mirror tile** (input Coords) → Color ramp → Output > Base color.

## Flipbook

Plays a grid of frames (sprite sheet) as animation.

*Works in: material, object shader, camera effect*

**Inputs**

| Input | Type | When not connected |
|---|---|---|
| Coords | vector | the object's UV |

**Parameters** (on the node or in the Inspector)

| Parameter | Default | Range | Choices |
|---|---|---|---|
| columns | 4 | 1..32 | - |
| rows | 4 | 1..32 | - |
| frames per second | 8 | 0..60 | - |

**Output**: vector

> Feeds **Vector / Coords / UV** inputs of textures and noises, or **Normal** of the output. For one channel pick the X/Y/Z conversion.

Example: Texture coordinate (World) → Mapping (scale 4) → **Flipbook** (input Coords) → Color ramp → Output > Base color.

## Distort coords

Wobbles coordinates with noise (heat haze, water).

*Works in: material, object shader, camera effect*

**Inputs**

| Input | Type | When not connected |
|---|---|---|
| Coords | vector | the object's UV |
| Amount | number | `0.05` |

**Parameters** (on the node or in the Inspector)

| Parameter | Default | Range | Choices |
|---|---|---|---|
| scale | 4 | 0.1..100 | - |

**Output**: vector

> Feeds **Vector / Coords / UV** inputs of textures and noises, or **Normal** of the output. For one channel pick the X/Y/Z conversion.

Example: Texture coordinate (World) → Mapping (scale 4) → **Distort coords** (input Coords) → Color ramp → Output > Base color.

## Rotate around axis

Rotates a vector around an axis by an angle (radians).

*Works in: material, object shader, camera effect*

**Inputs**

| Input | Type | When not connected |
|---|---|---|
| Vector | vector | `1.0, 0.0, 0.0` |
| Axis | vector | `0.0, 1.0, 0.0` |

**Parameters** (on the node or in the Inspector)

| Parameter | Default | Range | Choices |
|---|---|---|---|
| angle | 0 | -6.3..6.3 | - |

**Output**: vector

> Feeds **Vector / Coords / UV** inputs of textures and noises, or **Normal** of the output. For one channel pick the X/Y/Z conversion.

Example: Texture coordinate (World) → Mapping (scale 4) → **Rotate around axis** (input Vector) → Color ramp → Output > Base color.

## Refract

Bends a direction through a surface (glass, water).

*Works in: material, object shader, camera effect*

**Inputs**

| Input | Type | When not connected |
|---|---|---|
| Incident | number | `0.0, -1.0, 0.0` |
| Normal | vector | `0.0, 1.0, 0.0` |

**Parameters** (on the node or in the Inspector)

| Parameter | Default | Range | Choices |
|---|---|---|---|
| index ratio | 0.75 | 0.1..2 | - |

**Output**: vector

> Feeds **Vector / Coords / UV** inputs of textures and noises, or **Normal** of the output. For one channel pick the X/Y/Z conversion.

Example: Noise texture (Factor) → **Refract** (input Incident) → Mix colour (Factor).

## Twirl coords

Swirls coordinates around the centre.

*Works in: material, object shader, camera effect*

**Inputs**

| Input | Type | When not connected |
|---|---|---|
| Coords | vector | the object's UV |

**Parameters** (on the node or in the Inspector)

| Parameter | Default | Range | Choices |
|---|---|---|---|
| amount | 3 | -20..20 | - |

**Output**: vector

> Feeds **Vector / Coords / UV** inputs of textures and noises, or **Normal** of the output. For one channel pick the X/Y/Z conversion.

Example: Texture coordinate (World) → Mapping (scale 4) → **Twirl coords** (input Coords) → Color ramp → Output > Base color.

## Vector math

Operations on vectors: cross, dot, project, reflect, distance, normalize...

*Works in: material, object shader, camera effect*

**Inputs**

| Input | Type | When not connected |
|---|---|---|
| A | number | `0.0` |
| B | number | `1.0` |

**Parameters** (on the node or in the Inspector)

| Parameter | Default | Range | Choices |
|---|---|---|---|
| operation | 0 | 0..16 | 0 Add, 1 Subtract, 2 Multiply, 3 Divide, 4 Cross product, 5 Project, 6 Reflect, 7 Dot product, 8 Distance, 9 Length, 10 Scale, 11 Normalize, 12 Absolute, 13 Minimum, 14 Maximum, 15 Floor, 16 Fraction |

**Output**: vector

> Feeds **Vector / Coords / UV** inputs of textures and noises, or **Normal** of the output. For one channel pick the X/Y/Z conversion.

Example: Noise texture (Factor) → **Vector math** (input A) → Mix colour (Factor).

## Separate RGB

Splits a colour into R, G and B.

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

Example: Image texture (Colour) → **Separate RGB** (input Colour) → Output > Base color.

## Separate XYZ

Splits a vector into X, Y and Z.

*Works in: material, object shader, camera effect*

**Inputs**

| Input | Type | When not connected |
|---|---|---|
| Vector | vector | world position |

**Outputs**

| Output | Type |
|---|---|
| Vector | vector |
| X | number |
| Y | number |
| Z | number |

> Feeds **Vector / Coords / UV** inputs of textures and noises, or **Normal** of the output. For one channel pick the X/Y/Z conversion.

Example: Texture coordinate (World) → Mapping (scale 4) → **Separate XYZ** (input Vector) → Color ramp → Output > Base color.

## Combine XYZ

Builds a vector from three numbers.

*Works in: material, object shader, camera effect*

**Inputs**

| Input | Type | When not connected |
|---|---|---|
| X | number | `0.0` |
| Y | number | `0.0` |
| Z | number | `0.0` |

**Outputs**

| Output | Type |
|---|---|
| Vector | vector |
| Length | number |
| Normalized | vector |

> Feeds **Vector / Coords / UV** inputs of textures and noises, or **Normal** of the output. For one channel pick the X/Y/Z conversion.

Example: Noise texture (Factor) → **Combine XYZ** (input X) → Mix colour (Factor).

## Mapping

Moves, turns and scales texture coordinates.

*Works in: material, object shader, camera effect*

**Inputs**

| Input | Type | When not connected |
|---|---|---|
| Vector | vector | the object's UV |

**Parameters** (on the node or in the Inspector)

| Parameter | Default | Range | Choices |
|---|---|---|---|
| location x | 0 | -100..100 | - |
| location y | 0 | -100..100 | - |
| location z | 0 | -100..100 | - |
| rotation | 0 | -1..1 | - |
| scale x | 1 | -100..100 | - |
| scale y | 1 | -100..100 | - |
| scale z | 1 | -100..100 | - |

**Outputs**

| Output | Type |
|---|---|
| Vector | vector |
| X | number |
| Y | number |
| Z | number |

> Feeds **Vector / Coords / UV** inputs of textures and noises, or **Normal** of the output. For one channel pick the X/Y/Z conversion.

Example: Texture coordinate (World) → Mapping (scale 4) → **Mapping** (input Vector) → Color ramp → Output > Base color.

## Jitter coords

Adds smooth noise to coordinates to break visible repetition.

*Works in: material, object shader, camera effect*

**Inputs**

| Input | Type | When not connected |
|---|---|---|
| Vector | vector | world position |
| Amount | number | `0.2` |

**Parameters** (on the node or in the Inspector)

| Parameter | Default | Range | Choices |
|---|---|---|---|
| scale | 4 | 0.01..100 | - |

**Outputs**

| Output | Type |
|---|---|
| Vector | vector |
| X | number |
| Y | number |
| Z | number |

> Feeds **Vector / Coords / UV** inputs of textures and noises, or **Normal** of the output. For one channel pick the X/Y/Z conversion.

Example: Texture coordinate (World) → Mapping (scale 4) → **Jitter coords** (input Vector) → Color ramp → Output > Base color.

