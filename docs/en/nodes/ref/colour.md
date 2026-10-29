---
outline: [2, 3]
---

# Colour

Color ramp, blend modes, HSV, colour correction.

Generated from the node library. Socket types: **number** (grey circle), **colour** (yellow circle), **vector** (purple diamond). Any output can feed any input; when the types differ, pick a conversion on the input (right-click it). See [how nodes connect](../wiring).

## Hue shift

Rotates the colours around the colour wheel.

*Works in: material, object shader, camera effect*

**Inputs**

| Input | Type | When not connected |
|---|---|---|
| Colour | colour | `0.8` |

**Parameters** (on the node or in the Inspector)

| Parameter | Default | Range | Choices |
|---|---|---|---|
| turns | 0 | -1..1 | - |

**Output**: colour

> Usually goes to **Output > Base color** or **Emission**, or to **A/B** of Mix colour. Into a number input it is converted to luminance.

Example: Image texture (Colour) → **Hue shift** (input Colour) → Output > Base color.

## Saturation

0 = grey, 1 = unchanged, above 1 = more vivid.

*Works in: material, object shader, camera effect*

**Inputs**

| Input | Type | When not connected |
|---|---|---|
| Colour | colour | `0.8` |

**Parameters** (on the node or in the Inspector)

| Parameter | Default | Range | Choices |
|---|---|---|---|
| amount | 1 | 0..3 | - |

**Output**: colour

> Usually goes to **Output > Base color** or **Emission**, or to **A/B** of Mix colour. Into a number input it is converted to luminance.

Example: Image texture (Colour) → **Saturation** (input Colour) → Output > Base color.

## Contrast

Pushes values away from mid grey.

*Works in: material, object shader, camera effect*

**Inputs**

| Input | Type | When not connected |
|---|---|---|
| Colour | colour | `0.5` |

**Parameters** (on the node or in the Inspector)

| Parameter | Default | Range | Choices |
|---|---|---|---|
| amount | 1 | 0..3 | - |

**Output**: colour

> Usually goes to **Output > Base color** or **Emission**, or to **A/B** of Mix colour. Into a number input it is converted to luminance.

Example: Image texture (Colour) → **Contrast** (input Colour) → Output > Base color.

## Brightness

Multiplies the colour.

*Works in: material, object shader, camera effect*

**Inputs**

| Input | Type | When not connected |
|---|---|---|
| Colour | colour | `0.5` |

**Parameters** (on the node or in the Inspector)

| Parameter | Default | Range | Choices |
|---|---|---|---|
| scale | 1 | 0..10 | - |

**Output**: colour

> Usually goes to **Output > Base color** or **Emission**, or to **A/B** of Mix colour. Into a number input it is converted to luminance.

Example: Image texture (Colour) → **Brightness** (input Colour) → Output > Base color.

## Gradient

Blends two colours by a 0..1 position.

*Works in: material, object shader, camera effect*

**Inputs**

| Input | Type | When not connected |
|---|---|---|
| Position | vector | `0.5` |
| From | colour | `0.05, 0.1, 0.3` |
| To | colour | `0.9, 0.6, 0.2` |

**Output**: colour

> Usually goes to **Output > Base color** or **Emission**, or to **A/B** of Mix colour. Into a number input it is converted to luminance.

Example: Texture coordinate (World) → Mapping (scale 4) → **Gradient** (input Position) → Color ramp → Output > Base color.

## To grey

Converts a colour to its brightness.

*Works in: material, object shader, camera effect*

**Inputs**

| Input | Type | When not connected |
|---|---|---|
| Colour | colour | `0.5` |

**Output**: colour

> Usually goes to **Output > Base color** or **Emission**, or to **A/B** of Mix colour. Into a number input it is converted to luminance.

Example: Image texture (Colour) → **To grey** (input Colour) → Output > Base color.

## Overlay

Overlay blend: multiplies darks and screens lights.

*Works in: material, object shader, camera effect*

**Inputs**

| Input | Type | When not connected |
|---|---|---|
| Base | number | `0.5` |
| Blend | number | `0.5` |

**Output**: colour

> Usually goes to **Output > Base color** or **Emission**, or to **A/B** of Mix colour. Into a number input it is converted to luminance.

Example: Noise texture (Factor) → **Overlay** (input Base) → Mix colour (Factor).

## Screen blend

Screen blend: always lightens.

*Works in: material, object shader, camera effect*

**Inputs**

| Input | Type | When not connected |
|---|---|---|
| Base | number | `0.5` |
| Blend | number | `0.5` |

**Output**: colour

> Usually goes to **Output > Base color** or **Emission**, or to **A/B** of Mix colour. Into a number input it is converted to luminance.

Example: Noise texture (Factor) → **Screen blend** (input Base) → Mix colour (Factor).

## Colourize

Tints the brightness of a colour with another colour.

*Works in: material, object shader, camera effect*

**Inputs**

| Input | Type | When not connected |
|---|---|---|
| Value | number | `0.5` |
| Tint | colour | `1.0, 0.7, 0.4` |

**Output**: colour

> Usually goes to **Output > Base color** or **Emission**, or to **A/B** of Mix colour. Into a number input it is converted to luminance.

Example: Noise texture (Factor) → **Colourize** (input Value) → Mix colour (Factor).

## Ramp 3 colours

Maps 0..1 to a three-colour gradient.

*Works in: material, object shader, camera effect*

**Inputs**

| Input | Type | When not connected |
|---|---|---|
| Position | vector | `0.5` |
| Low | colour | `0.02, 0.03, 0.15` |
| Middle | colour | `0.8, 0.2, 0.3` |
| High | colour | `1.0, 0.9, 0.4` |

**Output**: colour

> Usually goes to **Output > Base color** or **Emission**, or to **A/B** of Mix colour. Into a number input it is converted to luminance.

Example: Texture coordinate (World) → Mapping (scale 4) → **Ramp 3 colours** (input Position) → Color ramp → Output > Base color.

## Gamma

Gamma correction.

*Works in: material, object shader, camera effect*

**Inputs**

| Input | Type | When not connected |
|---|---|---|
| Colour | colour | `0.5` |

**Parameters** (on the node or in the Inspector)

| Parameter | Default | Range | Choices |
|---|---|---|---|
| gamma | 2.2 | 0.1..5 | - |

**Output**: colour

> Usually goes to **Output > Base color** or **Emission**, or to **A/B** of Mix colour. Into a number input it is converted to luminance.

Example: Image texture (Colour) → **Gamma** (input Colour) → Output > Base color.

## Exposure

Brightens or darkens by photographic stops.

*Works in: material, object shader, camera effect*

**Inputs**

| Input | Type | When not connected |
|---|---|---|
| Colour | colour | `0.5` |

**Parameters** (on the node or in the Inspector)

| Parameter | Default | Range | Choices |
|---|---|---|---|
| stops | 0 | -8..8 | - |

**Output**: colour

> Usually goes to **Output > Base color** or **Emission**, or to **A/B** of Mix colour. Into a number input it is converted to luminance.

Example: Image texture (Colour) → **Exposure** (input Colour) → Output > Base color.

## Levels

Sets the black and white points.

*Works in: material, object shader, camera effect*

**Inputs**

| Input | Type | When not connected |
|---|---|---|
| Colour | colour | `0.5` |

**Parameters** (on the node or in the Inspector)

| Parameter | Default | Range | Choices |
|---|---|---|---|
| black | 0 | 0..1 | - |
| white | 1 | 0..2 | - |

**Output**: colour

> Usually goes to **Output > Base color** or **Emission**, or to **A/B** of Mix colour. Into a number input it is converted to luminance.

Example: Image texture (Colour) → **Levels** (input Colour) → Output > Base color.

## Threshold

Black and white by a brightness threshold.

*Works in: material, object shader, camera effect*

**Inputs**

| Input | Type | When not connected |
|---|---|---|
| Colour | colour | `0.5` |

**Parameters** (on the node or in the Inspector)

| Parameter | Default | Range | Choices |
|---|---|---|---|
| level | 0.5 | 0..1 | - |

**Output**: colour

> Usually goes to **Output > Base color** or **Emission**, or to **A/B** of Mix colour. Into a number input it is converted to luminance.

Example: Image texture (Colour) → **Threshold** (input Colour) → Output > Base color.

## Tone map

Filmic (ACES) tone mapping of HDR colour.

*Works in: material, object shader, camera effect*

**Inputs**

| Input | Type | When not connected |
|---|---|---|
| Colour | colour | `0.5` |

**Output**: colour

> Usually goes to **Output > Base color** or **Emission**, or to **A/B** of Mix colour. Into a number input it is converted to luminance.

Example: Image texture (Colour) → **Tone map** (input Colour) → Output > Base color.

## Color ramp

Maps 0..1 to a gradient of four colour stops placed at four positions.

*Works in: material, object shader, camera effect*

**Inputs**

| Input | Type | When not connected |
|---|---|---|
| Position | vector | `0.5` |
| Stop 1 | colour | `0.02, 0.02, 0.05` |
| Stop 2 | colour | `0.7, 0.15, 0.1` |
| Stop 3 | colour | `1.0, 0.7, 0.2` |
| Stop 4 | colour | `1.0, 1.0, 0.9` |

**Parameters** (on the node or in the Inspector)

| Parameter | Default | Range | Choices |
|---|---|---|---|
| position 1 | 0 | 0..1 | - |
| position 2 | 0.33 | 0..1 | - |
| position 3 | 0.66 | 0..1 | - |
| position 4 | 1 | 0..1 | - |
| interpolation | 0 | 0..2 | 0 Linear, 1 Constant, 2 Ease |

**Outputs**

| Output | Type |
|---|---|
| Colour | colour |
| Alpha | number |

> Usually goes to **Output > Base color** or **Emission**, or to **A/B** of Mix colour. Into a number input it is converted to luminance.

Example: Texture coordinate (World) → Mapping (scale 4) → **Color ramp** (input Position) → Color ramp → Output > Base color.

## Blend colours

Photoshop-style blend modes: 0 mix, 1 multiply, 2 screen, 3 overlay, 4 add, 5 subtract, 6 difference, 7 darken, 8 lighten, 9 soft light, 10 dodge, 11 burn, 12 hard light.

*Works in: material, object shader, camera effect*

**Inputs**

| Input | Type | When not connected |
|---|---|---|
| A | number | `0.5` |
| B | number | `0.5` |
| Factor | number | `1.0` |

**Parameters** (on the node or in the Inspector)

| Parameter | Default | Range | Choices |
|---|---|---|---|
| mode | 0 | 0..12 | - |

**Outputs**

| Output | Type |
|---|---|
| Colour | colour |
| R | number |
| G | number |
| B | number |
| Luma | number |

> Usually goes to **Output > Base color** or **Emission**, or to **A/B** of Mix colour. Into a number input it is converted to luminance.

Example: Noise texture (Factor) → **Blend colours** (input A) → Mix colour (Factor).

## HSV adjust

Shifts hue and scales saturation and brightness.

*Works in: material, object shader, camera effect*

**Inputs**

| Input | Type | When not connected |
|---|---|---|
| Colour | colour | `0.5` |

**Parameters** (on the node or in the Inspector)

| Parameter | Default | Range | Choices |
|---|---|---|---|
| hue shift | 0 | -0.5..0.5 | - |
| saturation | 1 | 0..3 | - |
| value | 1 | 0..3 | - |

**Outputs**

| Output | Type |
|---|---|
| Colour | colour |
| R | number |
| G | number |
| B | number |
| Luma | number |

> Usually goes to **Output > Base color** or **Emission**, or to **A/B** of Mix colour. Into a number input it is converted to luminance.

Example: Image texture (Colour) → **HSV adjust** (input Colour) → Output > Base color.

## HSV to RGB

Builds a colour from hue, saturation and value.

*Works in: material, object shader, camera effect*

**Inputs**

| Input | Type | When not connected |
|---|---|---|
| Hue | number | `0.0` |
| Saturation | number | `1.0` |
| Value | number | `1.0` |

**Output**: colour

> Usually goes to **Output > Base color** or **Emission**, or to **A/B** of Mix colour. Into a number input it is converted to luminance.

Example: Noise texture (Factor) → **HSV to RGB** (input Hue) → Mix colour (Factor).

## RGB to HSV

Splits a colour into hue (red), saturation (green), value (blue).

*Works in: material, object shader, camera effect*

**Inputs**

| Input | Type | When not connected |
|---|---|---|
| Colour | colour | `0.5` |

**Output**: colour

> Usually goes to **Output > Base color** or **Emission**, or to **A/B** of Mix colour. Into a number input it is converted to luminance.

Example: Image texture (Colour) → **RGB to HSV** (input Colour) → Output > Base color.

## Lift gamma gain

Classic three-way colour correction.

*Works in: material, object shader, camera effect*

**Inputs**

| Input | Type | When not connected |
|---|---|---|
| Colour | colour | `0.5` |

**Parameters** (on the node or in the Inspector)

| Parameter | Default | Range | Choices |
|---|---|---|---|
| lift | 0 | -0.5..0.5 | - |
| gamma | 1 | 0.1..4 | - |
| gain | 1 | 0..4 | - |

**Output**: colour

> Usually goes to **Output > Base color** or **Emission**, or to **A/B** of Mix colour. Into a number input it is converted to luminance.

Example: Image texture (Colour) → **Lift gamma gain** (input Colour) → Output > Base color.

## Colour temperature

Warm (orange) or cool (blue) tint.

*Works in: material, object shader, camera effect*

**Inputs**

| Input | Type | When not connected |
|---|---|---|
| Colour | colour | `0.5` |

**Parameters** (on the node or in the Inspector)

| Parameter | Default | Range | Choices |
|---|---|---|---|
| warmth | 0 | -1..1 | - |

**Output**: colour

> Usually goes to **Output > Base color** or **Emission**, or to **A/B** of Mix colour. Into a number input it is converted to luminance.

Example: Image texture (Colour) → **Colour temperature** (input Colour) → Output > Base color.

## Colour key

White where the colour is close to the key colour.

*Works in: material, object shader, camera effect*

**Inputs**

| Input | Type | When not connected |
|---|---|---|
| Colour | colour | `0.5` |

**Parameters** (on the node or in the Inspector)

| Parameter | Default | Range | Choices |
|---|---|---|---|
| colour | 1.00, 0.00, 0.00 | 0..1 | - |
| tolerance | 0.3 | 0..1.75 | - |
| softness | 0.1 | 0..1 | - |

**Output**: colour

> Usually goes to **Output > Base color** or **Emission**, or to **A/B** of Mix colour. Into a number input it is converted to luminance.

Example: Image texture (Colour) → **Colour key** (input Colour) → Output > Base color.

## S-curve

Contrast curve that keeps black and white fixed.

*Works in: material, object shader, camera effect*

**Inputs**

| Input | Type | When not connected |
|---|---|---|
| Value | number | `0.5` |

**Parameters** (on the node or in the Inspector)

| Parameter | Default | Range | Choices |
|---|---|---|---|
| amount | 0.7 | 0.05..0.95 | - |

**Output**: colour

> Usually goes to **Output > Base color** or **Emission**, or to **A/B** of Mix colour. Into a number input it is converted to luminance.

Example: Noise texture (Factor) → **S-curve** (input Value) → Mix colour (Factor).

## Bias curve

Bends mid-tones towards dark or light.

*Works in: material, object shader, camera effect*

**Inputs**

| Input | Type | When not connected |
|---|---|---|
| Value | number | `0.5` |

**Parameters** (on the node or in the Inspector)

| Parameter | Default | Range | Choices |
|---|---|---|---|
| bias | 0.5 | 0.01..0.99 | - |

**Output**: colour

> Usually goes to **Output > Base color** or **Emission**, or to **A/B** of Mix colour. Into a number input it is converted to luminance.

Example: Noise texture (Factor) → **Bias curve** (input Value) → Mix colour (Factor).

## Mix colour

Blends A and B with a mode (mix, multiply, overlay, screen...) by a factor.

*Works in: material, object shader, camera effect*

**Inputs**

| Input | Type | When not connected |
|---|---|---|
| Factor | number | `0.5` |
| A | number | `0.2` |
| B | number | `0.8` |

**Parameters** (on the node or in the Inspector)

| Parameter | Default | Range | Choices |
|---|---|---|---|
| blend mode | 0 | 0..12 | 0 Mix, 1 Multiply, 2 Screen, 3 Overlay, 4 Add, 5 Subtract, 6 Difference, 7 Darken, 8 Lighten, 9 Soft light, 10 Colour dodge, 11 Colour burn, 12 Hard light |
| clamp result | 0 | 0..1 | 0 Off, 1 Clamp 0..1 |

**Outputs**

| Output | Type |
|---|---|
| Colour | colour |
| R | number |
| G | number |
| B | number |
| Luma | number |

> Usually goes to **Output > Base color** or **Emission**, or to **A/B** of Mix colour. Into a number input it is converted to luminance.

Example: Noise texture (Factor) → **Mix colour** (input Factor) → Mix colour (Factor).

## Hue saturation value

Shifts hue and scales saturation and brightness.

*Works in: material, object shader, camera effect*

**Inputs**

| Input | Type | When not connected |
|---|---|---|
| Colour | colour | `0.8, 0.4, 0.2` |
| Hue | number | `0.5` |
| Saturation | number | `1.0` |
| Value | number | `1.0` |
| Factor | number | `1.0` |

**Outputs**

| Output | Type |
|---|---|
| Colour | colour |
| R | number |
| G | number |
| B | number |
| Luma | number |

> Usually goes to **Output > Base color** or **Emission**, or to **A/B** of Mix colour. Into a number input it is converted to luminance.

Example: Image texture (Colour) → **Hue saturation value** (input Colour) → Output > Base color.

## Invert

Inverts a colour by a factor.

*Works in: material, object shader, camera effect*

**Inputs**

| Input | Type | When not connected |
|---|---|---|
| Colour | colour | `0.8` |
| Factor | number | `1.0` |

**Outputs**

| Output | Type |
|---|---|
| Colour | colour |
| R | number |
| G | number |
| B | number |
| Luma | number |

> Usually goes to **Output > Base color** or **Emission**, or to **A/B** of Mix colour. Into a number input it is converted to luminance.

Example: Image texture (Colour) → **Invert** (input Colour) → Output > Base color.

## RGB curves

Tone curve for colours through five points.

*Works in: material, object shader, camera effect*

**Inputs**

| Input | Type | When not connected |
|---|---|---|
| Colour | colour | `0.5` |
| Factor | number | `1.0` |

**Parameters** (on the node or in the Inspector)

| Parameter | Default | Range | Choices |
|---|---|---|---|
| y at 0 | 0 | -1..2 | - |
| y at 0.25 | 0.22 | -1..2 | - |
| y at 0.5 | 0.5 | -1..2 | - |
| y at 0.75 | 0.78 | -1..2 | - |
| y at 1 | 1 | -1..2 | - |

**Outputs**

| Output | Type |
|---|---|
| Colour | colour |
| R | number |
| G | number |
| B | number |
| Luma | number |

> Usually goes to **Output > Base color** or **Emission**, or to **A/B** of Mix colour. Into a number input it is converted to luminance.

Example: Image texture (Colour) → **RGB curves** (input Colour) → Output > Base color.

## Bright contrast

Brightness and contrast as inputs.

*Works in: material, object shader, camera effect*

**Inputs**

| Input | Type | When not connected |
|---|---|---|
| Colour | colour | `0.5` |
| Bright | number | `0.0` |
| Contrast | number | `0.0` |

**Outputs**

| Output | Type |
|---|---|
| Colour | colour |
| R | number |
| G | number |
| B | number |
| Luma | number |

> Usually goes to **Output > Base color** or **Emission**, or to **A/B** of Mix colour. Into a number input it is converted to luminance.

Example: Image texture (Colour) → **Bright contrast** (input Colour) → Output > Base color.

