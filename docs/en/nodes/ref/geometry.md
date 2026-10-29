---
outline: [2, 3]
---

# Geometry

Position, normal, height, direction masks, vertex offsets.

Generated from the node library. Socket types: **number** (grey circle), **colour** (yellow circle), **vector** (purple diamond). Any output can feed any input; when the types differ, pick a conversion on the input (right-click it). See [how nodes connect](../wiring).

## UV

Texture coordinates of the surface (0..1).

*Works in: material, object shader, camera effect*

**Outputs**

| Output | Type |
|---|---|
| UV | vector |
| U | number |
| V | number |

> Feeds **Vector / Coords / UV** inputs of textures and noises, or **Normal** of the output. For one channel pick the X/Y/Z conversion.

## World position

Position of the pixel in the world.

*Works in: material, object shader*

**Outputs**

| Output | Type |
|---|---|
| Vector | vector |
| X | number |
| Y | number |
| Z | number |

> Feeds **Vector / Coords / UV** inputs of textures and noises, or **Normal** of the output. For one channel pick the X/Y/Z conversion.

## Normal

Direction the surface faces.

*Works in: material, object shader*

**Outputs**

| Output | Type |
|---|---|
| Normal | vector |
| Up facing | number |
| Down facing | number |

> Feeds **Vector / Coords / UV** inputs of textures and noises, or **Normal** of the output. For one channel pick the X/Y/Z conversion.

## Height

World height (Y) of the pixel.

*Works in: material, object shader*

**Output**: number

> Use it as a mask: into **Factor** of Mix colour, **Position** of a Color ramp, **Roughness / Metallic / Alpha / Height** of the output, or **A/B** of Math.

## Slope mask

1 on flat ground, 0 on steep slopes (between the two thresholds).

*Works in: material, object shader*

**Parameters** (on the node or in the Inspector)

| Parameter | Default | Range | Choices |
|---|---|---|---|
| from | 0.5 | 0..1 | - |
| to | 0.9 | 0..1 | - |

**Output**: number

> Use it as a mask: into **Factor** of Mix colour, **Position** of a Color ramp, **Roughness / Metallic / Alpha / Height** of the output, or **A/B** of Math.

## Camera distance

0 at the camera, 1 at the chosen range (fog, level of detail).

*Works in: material, object shader*

**Parameters** (on the node or in the Inspector)

| Parameter | Default | Range | Choices |
|---|---|---|---|
| range | 50 | 1..1000 | - |

**Output**: number

> Use it as a mask: into **Factor** of Mix colour, **Position** of a Color ramp, **Roughness / Metallic / Alpha / Height** of the output, or **A/B** of Math.

## View direction

Unit vector from the surface to the camera.

*Works in: material, object shader*

**Outputs**

| Output | Type |
|---|---|
| Vector | vector |
| X | number |
| Y | number |
| Z | number |

> Feeds **Vector / Coords / UV** inputs of textures and noises, or **Normal** of the output. For one channel pick the X/Y/Z conversion.

## Reflection

The view direction mirrored by the surface normal.

*Works in: material, object shader*

**Outputs**

| Output | Type |
|---|---|
| Vector | vector |
| X | number |
| Y | number |
| Z | number |

> Feeds **Vector / Coords / UV** inputs of textures and noises, or **Normal** of the output. For one channel pick the X/Y/Z conversion.

## Camera position

World position of the camera.

*Works in: material, object shader*

**Outputs**

| Output | Type |
|---|---|
| Vector | vector |
| X | number |
| Y | number |
| Z | number |

> Feeds **Vector / Coords / UV** inputs of textures and noises, or **Normal** of the output. For one channel pick the X/Y/Z conversion.

## Object position

World position of the object's origin.

*Works in: material, object shader*

**Outputs**

| Output | Type |
|---|---|
| Vector | vector |
| X | number |
| Y | number |
| Z | number |

> Feeds **Vector / Coords / UV** inputs of textures and noises, or **Normal** of the output. For one channel pick the X/Y/Z conversion.

## Object random

A random number that is different for every object but constant over time.

*Works in: material, object shader*

**Output**: vector

> Feeds **Vector / Coords / UV** inputs of textures and noises, or **Normal** of the output. For one channel pick the X/Y/Z conversion.

## Screen position

Pixel position on the screen (0..1).

*Works in: material, object shader*

**Output**: vector

> Feeds **Vector / Coords / UV** inputs of textures and noises, or **Normal** of the output. For one channel pick the X/Y/Z conversion.

## Direction mask

White where the surface faces the chosen direction (x, y, z). Top = (0,1,0).

*Works in: material, object shader*

**Parameters** (on the node or in the Inspector)

| Parameter | Default | Range | Choices |
|---|---|---|---|
| x | 0 | -1..1 | - |
| y | 1 | -1..1 | - |
| z | 0 | -1..1 | - |
| threshold | 0.5 | -1..1 | - |
| softness | 0.2 | 0.001..2 | - |

**Output**: vector

> Feeds **Vector / Coords / UV** inputs of textures and noises, or **Normal** of the output. For one channel pick the X/Y/Z conversion.

## Surface sides

Red = top faces, green = side faces, blue = bottom faces. Split with Red / Green / Blue nodes.

*Works in: material, object shader*

**Output**: vector

> Feeds **Vector / Coords / UV** inputs of textures and noises, or **Normal** of the output. For one channel pick the X/Y/Z conversion.

## Triplanar weights

How strongly each world axis projection (X, Y, Z) applies on this surface.

*Works in: material, object shader*

**Parameters** (on the node or in the Inspector)

| Parameter | Default | Range | Choices |
|---|---|---|---|
| sharpness | 4 | 1..32 | - |

**Output**: vector

> Feeds **Vector / Coords / UV** inputs of textures and noises, or **Normal** of the output. For one channel pick the X/Y/Z conversion.

## Local position

Position relative to the object's origin.

*Works in: material, object shader*

**Outputs**

| Output | Type |
|---|---|
| Vector | vector |
| X | number |
| Y | number |
| Z | number |

> Feeds **Vector / Coords / UV** inputs of textures and noises, or **Normal** of the output. For one channel pick the X/Y/Z conversion.

## Object distance

0 at the object's centre, 1 at the chosen range.

*Works in: material, object shader*

**Parameters** (on the node or in the Inspector)

| Parameter | Default | Range | Choices |
|---|---|---|---|
| range | 5 | 0.1..500 | - |

**Output**: vector

> Feeds **Vector / Coords / UV** inputs of textures and noises, or **Normal** of the output. For one channel pick the X/Y/Z conversion.

## World gradient

A 0..1 ramp along a world direction between two values (fog by height, snow line).

*Works in: material, object shader*

**Parameters** (on the node or in the Inspector)

| Parameter | Default | Range | Choices |
|---|---|---|---|
| x | 0 | -1..1 | - |
| y | 1 | -1..1 | - |
| z | 0 | -1..1 | - |
| from | 0 | -1000..1000 | - |
| to | 4 | -1000..1000 | - |

**Output**: vector

> Feeds **Vector / Coords / UV** inputs of textures and noises, or **Normal** of the output. For one channel pick the X/Y/Z conversion.

## Planar UV

Texture coordinates projected from world space along an axis.

*Works in: material, object shader*

**Parameters** (on the node or in the Inspector)

| Parameter | Default | Range | Choices |
|---|---|---|---|
| axis | 1 | 0..2 | - |
| scale | 1 | 0.01..100 | - |

**Output**: vector

> Feeds **Vector / Coords / UV** inputs of textures and noises, or **Normal** of the output. For one channel pick the X/Y/Z conversion.

## Normal UV

Spherical coordinates from the normal (for sky-like mapping).

*Works in: material, object shader*

**Output**: vector

> Feeds **Vector / Coords / UV** inputs of textures and noises, or **Normal** of the output. For one channel pick the X/Y/Z conversion.

## Wave offset

Vertex offset for waves. Connect to Shader output > Vertex offset.

*Works in: material, object shader*

**Parameters** (on the node or in the Inspector)

| Parameter | Default | Range | Choices |
|---|---|---|---|
| height | 0.1 | 0..5 | - |
| frequency | 2 | 0..30 | - |
| speed | 1 | -10..10 | - |

**Output**: vector

> Feeds **Vector / Coords / UV** inputs of textures and noises, or **Normal** of the output. For one channel pick the X/Y/Z conversion.

## Wind sway

Vertex offset that bends things in the wind, more at the top. Connect to Vertex offset.

*Works in: material, object shader*

**Parameters** (on the node or in the Inspector)

| Parameter | Default | Range | Choices |
|---|---|---|---|
| strength | 0.15 | 0..3 | - |
| speed | 1.5 | 0..20 | - |
| scale | 0.8 | 0..10 | - |

**Output**: vector

> Feeds **Vector / Coords / UV** inputs of textures and noises, or **Normal** of the output. For one channel pick the X/Y/Z conversion.

## Texture coordinate

Every coordinate system at once: UV, object, world, normal, reflection and screen.

*Works in: material, object shader*

**Outputs**

| Output | Type |
|---|---|
| UV | vector |
| Object | vector |
| World | vector |
| Normal | vector |
| Reflection | vector |
| Screen | vector |

> Feeds **Vector / Coords / UV** inputs of textures and noises, or **Normal** of the output. For one channel pick the X/Y/Z conversion.

## Geometry info

Position, normal, view direction, height, back side and slope of the surface point.

*Works in: material, object shader*

**Outputs**

| Output | Type |
|---|---|
| Position | vector |
| Normal | vector |
| View | vector |
| Height | number |
| Backfacing | number |
| Slope | number |

> Feeds **Vector / Coords / UV** inputs of textures and noises, or **Normal** of the output. For one channel pick the X/Y/Z conversion.

