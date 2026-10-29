---
outline: [2, 3]
---

# Light

Sun, shadows, highlights, rim light, glow (Emission).

Generated from the node library. Socket types: **number** (grey circle), **colour** (yellow circle), **vector** (purple diamond). Any output can feed any input; when the types differ, pick a conversion on the input (right-click it). See [how nodes connect](../wiring).

## Fresnel

Brighter at grazing angles (rim light, glass, water edges).

*Works in: material, object shader*

**Inputs**

| Input | Type | When not connected |
|---|---|---|
| Power | number | `3.0` |

**Outputs**

| Output | Type |
|---|---|
| Fresnel | number |
| Facing | number |

> Use it as a mask: into **Factor** of Mix colour, **Position** of a Color ramp, **Roughness / Metallic / Alpha / Height** of the output, or **A/B** of Math.

Example: Noise texture (Factor) → **Fresnel** (input Power) → Mix colour (Factor).

## Sun light

How much the surface faces the sun (0..1), without shadows.

*Works in: material, object shader*

**Output**: colour

> Usually goes to **Output > Base color** or **Emission**, or to **A/B** of Mix colour. Into a number input it is converted to luminance.

## Sky reflection

What the sky looks like in the mirror direction.

*Works in: material, object shader*

**Output**: colour

> Usually goes to **Output > Base color** or **Emission**, or to **A/B** of Mix colour. Into a number input it is converted to luminance.

## Sun direction

Unit vector pointing to the sun.

*Works in: material, object shader*

**Outputs**

| Output | Type |
|---|---|
| Vector | vector |
| X | number |
| Y | number |
| Z | number |

> Feeds **Vector / Coords / UV** inputs of textures and noises, or **Normal** of the output. For one channel pick the X/Y/Z conversion.

## Sun colour

Colour and strength of the sun.

*Works in: material, object shader*

**Output**: colour

> Usually goes to **Output > Base color** or **Emission**, or to **A/B** of Mix colour. Into a number input it is converted to luminance.

## Ambient light

Constant ambient light colour of the scene.

*Works in: material, object shader*

**Output**: colour

> Usually goes to **Output > Base color** or **Emission**, or to **A/B** of Mix colour. Into a number input it is converted to luminance.

## Shadow

1 in light, 0 in the sun shadow.

*Works in: material, object shader*

**Output**: colour

> Usually goes to **Output > Base color** or **Emission**, or to **A/B** of Mix colour. Into a number input it is converted to luminance.

## Lighting

Applies scene lighting (sun, shadows, lamps) to a colour.

*Works in: material, object shader*

**Inputs**

| Input | Type | When not connected |
|---|---|---|
| Base | number | `0.8` |

**Parameters** (on the node or in the Inspector)

| Parameter | Default | Range | Choices |
|---|---|---|---|
| roughness | 0.6 | 0..1 | - |

**Output**: colour

> Usually goes to **Output > Base color** or **Emission**, or to **A/B** of Mix colour. Into a number input it is converted to luminance.

Example: Noise texture (Factor) → **Lighting** (input Base) → Mix colour (Factor).

## Light angle

Dot product of the normal and the sun direction: 1 facing the sun, -1 facing away.

*Works in: material, object shader*

**Output**: number

> Use it as a mask: into **Factor** of Mix colour, **Position** of a Color ramp, **Roughness / Metallic / Alpha / Height** of the output, or **A/B** of Math.

## Half lambert

Soft wrap-around sun lighting (0..1) without a hard dark side.

*Works in: material, object shader*

**Parameters** (on the node or in the Inspector)

| Parameter | Default | Range | Choices |
|---|---|---|---|
| power | 1 | 0.1..4 | - |

**Output**: number

> Use it as a mask: into **Factor** of Mix colour, **Position** of a Color ramp, **Roughness / Metallic / Alpha / Height** of the output, or **A/B** of Math.

## Specular highlight

Blinn-Phong glint of the sun. Higher shininess = smaller, sharper spot.

*Works in: material, object shader*

**Parameters** (on the node or in the Inspector)

| Parameter | Default | Range | Choices |
|---|---|---|---|
| shininess | 64 | 1..512 | - |
| intensity | 1 | 0..10 | - |

**Output**: colour

> Usually goes to **Output > Base color** or **Emission**, or to **A/B** of Mix colour. Into a number input it is converted to luminance.

## Rim light

A coloured glow on the edges that face away from the camera. Plug into Emission.

*Works in: material, object shader*

**Parameters** (on the node or in the Inspector)

| Parameter | Default | Range | Choices |
|---|---|---|---|
| colour | 0.40, 0.70, 1.00 | 0..1 | - |
| power | 3 | 0.1..12 | - |
| intensity | 1 | 0..20 | - |

**Output**: colour

> Usually goes to **Output > Base color** or **Emission**, or to **A/B** of Mix colour. Into a number input it is converted to luminance.

## Backlight

Bright when the camera looks towards the sun through the surface (leaves, cloth, ice).

*Works in: material, object shader*

**Parameters** (on the node or in the Inspector)

| Parameter | Default | Range | Choices |
|---|---|---|---|
| sharpness | 6 | 1..64 | - |

**Output**: number

> Use it as a mask: into **Factor** of Mix colour, **Position** of a Color ramp, **Roughness / Metallic / Alpha / Height** of the output, or **A/B** of Math.

## Facing ratio

1 where the surface faces the camera, 0 on silhouettes.

*Works in: material, object shader*

**Output**: number

> Use it as a mask: into **Factor** of Mix colour, **Position** of a Color ramp, **Roughness / Metallic / Alpha / Height** of the output, or **A/B** of Math.

## Sun with shadow

Sun lighting amount (0..1) that respects shadows.

*Works in: material, object shader*

**Output**: number

> Use it as a mask: into **Factor** of Mix colour, **Position** of a Color ramp, **Roughness / Metallic / Alpha / Height** of the output, or **A/B** of Math.

## Sun lighting

Sun colour times the lighting amount, with shadows. Add Sky light for a full diffuse term.

*Works in: material, object shader*

**Output**: colour

> Usually goes to **Output > Base color** or **Emission**, or to **A/B** of Mix colour. Into a number input it is converted to luminance.

## Sky light

Ambient light from the sky in the direction of the normal.

*Works in: material, object shader*

**Output**: colour

> Usually goes to **Output > Base color** or **Emission**, or to **A/B** of Mix colour. Into a number input it is converted to luminance.

## Glow

A constant colour multiplied by an intensity. Plug into Emission to make things shine.

*Works in: material, object shader, camera effect*

**Parameters** (on the node or in the Inspector)

| Parameter | Default | Range | Choices |
|---|---|---|---|
| colour | 1.00, 0.60, 0.20 | 0..1 | - |
| intensity | 2 | 0..50 | - |

**Output**: colour

> Usually goes to **Output > Base color** or **Emission**, or to **A/B** of Mix colour. Into a number input it is converted to luminance.

## Pulsing glow

A glow that pulses over time.

*Works in: material, object shader, camera effect*

**Parameters** (on the node or in the Inspector)

| Parameter | Default | Range | Choices |
|---|---|---|---|
| colour | 1.00, 0.30, 0.10 | 0..1 | - |
| intensity | 3 | 0..50 | - |
| speed | 1 | 0..20 | - |

**Output**: colour

> Usually goes to **Output > Base color** or **Emission**, or to **A/B** of Mix colour. Into a number input it is converted to luminance.

## Layer weight

Fresnel and facing masks with a blend control; good for coatings and rims.

*Works in: material, object shader*

**Inputs**

| Input | Type | When not connected |
|---|---|---|
| Blend | number | `0.5` |

**Outputs**

| Output | Type |
|---|---|
| Fresnel | number |
| Facing | number |

> Use it as a mask: into **Factor** of Mix colour, **Position** of a Color ramp, **Roughness / Metallic / Alpha / Height** of the output, or **A/B** of Math.

Example: Noise texture (Factor) → **Layer weight** (input Blend) → Mix colour (Factor).

