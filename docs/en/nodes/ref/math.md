---
outline: [2, 3]
---

# Math

Arithmetic, curves, comparisons, transitions.

Generated from the node library. Socket types: **number** (grey circle), **colour** (yellow circle), **vector** (purple diamond). Any output can feed any input; when the types differ, pick a conversion on the input (right-click it). See [how nodes connect](../wiring).

## Add

A + B.

*Works in: material, object shader, camera effect*

**Inputs**

| Input | Type | When not connected |
|---|---|---|
| A | number | `0.0` |
| B | number | `0.0` |

**Output**: number

> Use it as a mask: into **Factor** of Mix colour, **Position** of a Color ramp, **Roughness / Metallic / Alpha / Height** of the output, or **A/B** of Math.

Example: Noise texture (Factor) → **Add** (input A) → Mix colour (Factor).

## Subtract

A - B.

*Works in: material, object shader, camera effect*

**Inputs**

| Input | Type | When not connected |
|---|---|---|
| A | number | `0.0` |
| B | number | `0.0` |

**Output**: number

> Use it as a mask: into **Factor** of Mix colour, **Position** of a Color ramp, **Roughness / Metallic / Alpha / Height** of the output, or **A/B** of Math.

Example: Noise texture (Factor) → **Subtract** (input A) → Mix colour (Factor).

## Multiply

A * B (tinting, masking).

*Works in: material, object shader, camera effect*

**Inputs**

| Input | Type | When not connected |
|---|---|---|
| A | number | `1.0` |
| B | number | `1.0` |

**Output**: number

> Use it as a mask: into **Factor** of Mix colour, **Position** of a Color ramp, **Roughness / Metallic / Alpha / Height** of the output, or **A/B** of Math.

Example: Noise texture (Factor) → **Multiply** (input A) → Mix colour (Factor).

## Divide

A / B (safe: never divides by zero).

*Works in: material, object shader, camera effect*

**Inputs**

| Input | Type | When not connected |
|---|---|---|
| A | number | `1.0` |
| B | number | `1.0` |

**Output**: number

> Use it as a mask: into **Factor** of Mix colour, **Position** of a Color ramp, **Roughness / Metallic / Alpha / Height** of the output, or **A/B** of Math.

Example: Noise texture (Factor) → **Divide** (input A) → Mix colour (Factor).

## Mix

Blends A and B by Factor (0 = A, 1 = B).

*Works in: material, object shader, camera effect*

**Inputs**

| Input | Type | When not connected |
|---|---|---|
| A | number | `0.0` |
| B | number | `1.0` |
| Factor | number | `0.5` |

**Output**: number

> Use it as a mask: into **Factor** of Mix colour, **Position** of a Color ramp, **Roughness / Metallic / Alpha / Height** of the output, or **A/B** of Math.

Example: Noise texture (Factor) → **Mix** (input A) → Mix colour (Factor).

## Power

Raises to a power (curves, sharpening masks).

*Works in: material, object shader, camera effect*

**Inputs**

| Input | Type | When not connected |
|---|---|---|
| Value | number | `0.5` |
| Exponent | number | `2.0` |

**Output**: number

> Use it as a mask: into **Factor** of Mix colour, **Position** of a Color ramp, **Roughness / Metallic / Alpha / Height** of the output, or **A/B** of Math.

Example: Noise texture (Factor) → **Power** (input Value) → Mix colour (Factor).

## Clamp

Limits a value to a range.

*Works in: material, object shader, camera effect*

**Inputs**

| Input | Type | When not connected |
|---|---|---|
| Value | number | `0.0` |

**Parameters** (on the node or in the Inspector)

| Parameter | Default | Range | Choices |
|---|---|---|---|
| min | 0 | -100..100 | - |
| max | 1 | -100..100 | - |

**Output**: number

> Use it as a mask: into **Factor** of Mix colour, **Position** of a Color ramp, **Roughness / Metallic / Alpha / Height** of the output, or **A/B** of Math.

Example: Noise texture (Factor) → **Clamp** (input Value) → Mix colour (Factor).

## One minus

1 - value (inverts a mask).

*Works in: material, object shader, camera effect*

**Inputs**

| Input | Type | When not connected |
|---|---|---|
| Value | number | `0.0` |

**Output**: number

> Use it as a mask: into **Factor** of Mix colour, **Position** of a Color ramp, **Roughness / Metallic / Alpha / Height** of the output, or **A/B** of Math.

Example: Noise texture (Factor) → **One minus** (input Value) → Mix colour (Factor).

## Sine

A sine wave remapped to 0..1.

*Works in: material, object shader, camera effect*

**Inputs**

| Input | Type | When not connected |
|---|---|---|
| Value | number | `0.0` |

**Parameters** (on the node or in the Inspector)

| Parameter | Default | Range | Choices |
|---|---|---|---|
| frequency | 1 | 0..100 | - |

**Output**: number

> Use it as a mask: into **Factor** of Mix colour, **Position** of a Color ramp, **Roughness / Metallic / Alpha / Height** of the output, or **A/B** of Math.

Example: Noise texture (Factor) → **Sine** (input Value) → Mix colour (Factor).

## Smoothstep

Smooth 0..1 transition between two edges.

*Works in: material, object shader, camera effect*

**Inputs**

| Input | Type | When not connected |
|---|---|---|
| Value | number | `0.5` |

**Parameters** (on the node or in the Inspector)

| Parameter | Default | Range | Choices |
|---|---|---|---|
| edge 0 | 0.3 | -10..10 | - |
| edge 1 | 0.7 | -10..10 | - |

**Output**: number

> Use it as a mask: into **Factor** of Mix colour, **Position** of a Color ramp, **Roughness / Metallic / Alpha / Height** of the output, or **A/B** of Math.

Example: Noise texture (Factor) → **Smoothstep** (input Value) → Mix colour (Factor).

## Abs

Absolute value.

*Works in: material, object shader, camera effect*

**Inputs**

| Input | Type | When not connected |
|---|---|---|
| Value | number | `0.0` |

**Output**: number

> Use it as a mask: into **Factor** of Mix colour, **Position** of a Color ramp, **Roughness / Metallic / Alpha / Height** of the output, or **A/B** of Math.

Example: Noise texture (Factor) → **Abs** (input Value) → Mix colour (Factor).

## Min

The smaller of two values.

*Works in: material, object shader, camera effect*

**Inputs**

| Input | Type | When not connected |
|---|---|---|
| A | number | `0.0` |
| B | number | `0.0` |

**Output**: number

> Use it as a mask: into **Factor** of Mix colour, **Position** of a Color ramp, **Roughness / Metallic / Alpha / Height** of the output, or **A/B** of Math.

Example: Noise texture (Factor) → **Min** (input A) → Mix colour (Factor).

## Max

The larger of two values.

*Works in: material, object shader, camera effect*

**Inputs**

| Input | Type | When not connected |
|---|---|---|
| A | number | `0.0` |
| B | number | `0.0` |

**Output**: number

> Use it as a mask: into **Factor** of Mix colour, **Position** of a Color ramp, **Roughness / Metallic / Alpha / Height** of the output, or **A/B** of Math.

Example: Noise texture (Factor) → **Max** (input A) → Mix colour (Factor).

## Fract

The fractional part (repeats 0..1).

*Works in: material, object shader, camera effect*

**Inputs**

| Input | Type | When not connected |
|---|---|---|
| Value | number | `0.5` |

**Output**: number

> Use it as a mask: into **Factor** of Mix colour, **Position** of a Color ramp, **Roughness / Metallic / Alpha / Height** of the output, or **A/B** of Math.

Example: Noise texture (Factor) → **Fract** (input Value) → Mix colour (Factor).

## Step

0 below the edge, 1 above it.

*Works in: material, object shader, camera effect*

**Inputs**

| Input | Type | When not connected |
|---|---|---|
| Value | number | `0.5` |

**Parameters** (on the node or in the Inspector)

| Parameter | Default | Range | Choices |
|---|---|---|---|
| edge | 0.5 | 0..1 | - |

**Output**: number

> Use it as a mask: into **Factor** of Mix colour, **Position** of a Color ramp, **Roughness / Metallic / Alpha / Height** of the output, or **A/B** of Math.

Example: Noise texture (Factor) → **Step** (input Value) → Mix colour (Factor).

## Remap

Maps a range to another range (not clamped).

*Works in: material, object shader, camera effect*

**Inputs**

| Input | Type | When not connected |
|---|---|---|
| Value | number | `0.5` |

**Parameters** (on the node or in the Inspector)

| Parameter | Default | Range | Choices |
|---|---|---|---|
| in min | 0 | -10..10 | - |
| in max | 1 | -10..10 | - |
| out min | 0 | -10..10 | - |
| out max | 1 | -10..10 | - |

**Output**: number

> Use it as a mask: into **Factor** of Mix colour, **Position** of a Color ramp, **Roughness / Metallic / Alpha / Height** of the output, or **A/B** of Math.

Example: Noise texture (Factor) → **Remap** (input Value) → Mix colour (Factor).

## Negate

Changes the sign.

*Works in: material, object shader, camera effect*

**Inputs**

| Input | Type | When not connected |
|---|---|---|
| Value | number | `0.0` |

**Output**: number

> Use it as a mask: into **Factor** of Mix colour, **Position** of a Color ramp, **Roughness / Metallic / Alpha / Height** of the output, or **A/B** of Math.

Example: Noise texture (Factor) → **Negate** (input Value) → Mix colour (Factor).

## Square root

Square root.

*Works in: material, object shader, camera effect*

**Inputs**

| Input | Type | When not connected |
|---|---|---|
| Value | number | `0.5` |

**Output**: number

> Use it as a mask: into **Factor** of Mix colour, **Position** of a Color ramp, **Roughness / Metallic / Alpha / Height** of the output, or **A/B** of Math.

Example: Noise texture (Factor) → **Square root** (input Value) → Mix colour (Factor).

## Floor

Rounds down.

*Works in: material, object shader, camera effect*

**Inputs**

| Input | Type | When not connected |
|---|---|---|
| Value | number | `0.5` |

**Output**: number

> Use it as a mask: into **Factor** of Mix colour, **Position** of a Color ramp, **Roughness / Metallic / Alpha / Height** of the output, or **A/B** of Math.

Example: Noise texture (Factor) → **Floor** (input Value) → Mix colour (Factor).

## Ceil

Rounds up.

*Works in: material, object shader, camera effect*

**Inputs**

| Input | Type | When not connected |
|---|---|---|
| Value | number | `0.5` |

**Output**: number

> Use it as a mask: into **Factor** of Mix colour, **Position** of a Color ramp, **Roughness / Metallic / Alpha / Height** of the output, or **A/B** of Math.

Example: Noise texture (Factor) → **Ceil** (input Value) → Mix colour (Factor).

## Round

Rounds to the nearest whole number.

*Works in: material, object shader, camera effect*

**Inputs**

| Input | Type | When not connected |
|---|---|---|
| Value | number | `0.5` |

**Output**: number

> Use it as a mask: into **Factor** of Mix colour, **Position** of a Color ramp, **Roughness / Metallic / Alpha / Height** of the output, or **A/B** of Math.

Example: Noise texture (Factor) → **Round** (input Value) → Mix colour (Factor).

## Sign

-1, 0 or 1.

*Works in: material, object shader, camera effect*

**Inputs**

| Input | Type | When not connected |
|---|---|---|
| Value | number | `0.0` |

**Output**: number

> Use it as a mask: into **Factor** of Mix colour, **Position** of a Color ramp, **Roughness / Metallic / Alpha / Height** of the output, or **A/B** of Math.

Example: Noise texture (Factor) → **Sign** (input Value) → Mix colour (Factor).

## Saturate

Clamps to 0..1.

*Works in: material, object shader, camera effect*

**Inputs**

| Input | Type | When not connected |
|---|---|---|
| Value | number | `0.0` |

**Output**: number

> Use it as a mask: into **Factor** of Mix colour, **Position** of a Color ramp, **Roughness / Metallic / Alpha / Height** of the output, or **A/B** of Math.

Example: Noise texture (Factor) → **Saturate** (input Value) → Mix colour (Factor).

## Modulo

The remainder of A / B (repeating).

*Works in: material, object shader, camera effect*

**Inputs**

| Input | Type | When not connected |
|---|---|---|
| A | number | `0.0` |
| B | number | `1.0` |

**Output**: number

> Use it as a mask: into **Factor** of Mix colour, **Position** of a Color ramp, **Roughness / Metallic / Alpha / Height** of the output, or **A/B** of Math.

Example: Noise texture (Factor) → **Modulo** (input A) → Mix colour (Factor).

## Multiply add

A * B + C in one node.

*Works in: material, object shader, camera effect*

**Inputs**

| Input | Type | When not connected |
|---|---|---|
| A | number | `1.0` |
| B | number | `1.0` |
| C | number | `0.0` |

**Output**: number

> Use it as a mask: into **Factor** of Mix colour, **Position** of a Color ramp, **Roughness / Metallic / Alpha / Height** of the output, or **A/B** of Math.

Example: Noise texture (Factor) → **Multiply add** (input A) → Mix colour (Factor).

## Cosine

A cosine wave remapped to 0..1.

*Works in: material, object shader, camera effect*

**Inputs**

| Input | Type | When not connected |
|---|---|---|
| Value | number | `0.0` |

**Parameters** (on the node or in the Inspector)

| Parameter | Default | Range | Choices |
|---|---|---|---|
| frequency | 1 | 0..100 | - |

**Output**: number

> Use it as a mask: into **Factor** of Mix colour, **Position** of a Color ramp, **Roughness / Metallic / Alpha / Height** of the output, or **A/B** of Math.

Example: Noise texture (Factor) → **Cosine** (input Value) → Mix colour (Factor).

## Triangle wave

A zig-zag wave 0..1..0.

*Works in: material, object shader, camera effect*

**Inputs**

| Input | Type | When not connected |
|---|---|---|
| Value | number | `0.0` |

**Parameters** (on the node or in the Inspector)

| Parameter | Default | Range | Choices |
|---|---|---|---|
| frequency | 1 | 0..100 | - |

**Output**: number

> Use it as a mask: into **Factor** of Mix colour, **Position** of a Color ramp, **Roughness / Metallic / Alpha / Height** of the output, or **A/B** of Math.

Example: Noise texture (Factor) → **Triangle wave** (input Value) → Mix colour (Factor).

## Sawtooth

A ramp that jumps back to 0.

*Works in: material, object shader, camera effect*

**Inputs**

| Input | Type | When not connected |
|---|---|---|
| Value | number | `0.0` |

**Parameters** (on the node or in the Inspector)

| Parameter | Default | Range | Choices |
|---|---|---|---|
| frequency | 1 | 0..100 | - |

**Output**: number

> Use it as a mask: into **Factor** of Mix colour, **Position** of a Color ramp, **Roughness / Metallic / Alpha / Height** of the output, or **A/B** of Math.

Example: Noise texture (Factor) → **Sawtooth** (input Value) → Mix colour (Factor).

## Square wave

Alternates between 0 and 1.

*Works in: material, object shader, camera effect*

**Inputs**

| Input | Type | When not connected |
|---|---|---|
| Value | number | `0.0` |

**Parameters** (on the node or in the Inspector)

| Parameter | Default | Range | Choices |
|---|---|---|---|
| frequency | 1 | 0..100 | - |

**Output**: number

> Use it as a mask: into **Factor** of Mix colour, **Position** of a Color ramp, **Roughness / Metallic / Alpha / Height** of the output, or **A/B** of Math.

Example: Noise texture (Factor) → **Square wave** (input Value) → Mix colour (Factor).

## Posterize

Reduces the number of distinct levels.

*Works in: material, object shader, camera effect*

**Inputs**

| Input | Type | When not connected |
|---|---|---|
| Value | number | `0.5` |

**Parameters** (on the node or in the Inspector)

| Parameter | Default | Range | Choices |
|---|---|---|---|
| steps | 4 | 2..64 | - |

**Output**: number

> Use it as a mask: into **Factor** of Mix colour, **Position** of a Color ramp, **Roughness / Metallic / Alpha / Height** of the output, or **A/B** of Math.

Example: Noise texture (Factor) → **Posterize** (input Value) → Mix colour (Factor).

## Greater than

1 where A > B, otherwise 0.

*Works in: material, object shader, camera effect*

**Inputs**

| Input | Type | When not connected |
|---|---|---|
| A | number | `0.0` |
| B | number | `0.5` |

**Output**: number

> Use it as a mask: into **Factor** of Mix colour, **Position** of a Color ramp, **Roughness / Metallic / Alpha / Height** of the output, or **A/B** of Math.

Example: Noise texture (Factor) → **Greater than** (input A) → Mix colour (Factor).

## Branch

If A > B returns the third input, otherwise the fourth.

*Works in: material, object shader, camera effect*

**Inputs**

| Input | Type | When not connected |
|---|---|---|
| A | number | `0.0` |
| B | number | `0.5` |
| If A > B | number | `1.0` |
| Otherwise | number | `0.0` |

**Output**: number

> Use it as a mask: into **Factor** of Mix colour, **Position** of a Color ramp, **Roughness / Metallic / Alpha / Height** of the output, or **A/B** of Math.

Example: Noise texture (Factor) → **Branch** (input A) → Mix colour (Factor).

## Height blend

A blend mask where the higher of two heights wins (rock through snow, moss in cracks).

*Works in: material, object shader, camera effect*

**Inputs**

| Input | Type | When not connected |
|---|---|---|
| Height A | number | `0.0` |
| Height B | number | `0.0` |
| Blend | number | `0.5` |

**Parameters** (on the node or in the Inspector)

| Parameter | Default | Range | Choices |
|---|---|---|---|
| sharpness | 0.2 | 0.001..2 | - |

**Output**: number

> Use it as a mask: into **Factor** of Mix colour, **Position** of a Color ramp, **Roughness / Metallic / Alpha / Height** of the output, or **A/B** of Math.

Example: Noise texture (Factor) → **Height blend** (input Height A) → Mix colour (Factor).

## Exponential

e raised to the value.

*Works in: material, object shader, camera effect*

**Inputs**

| Input | Type | When not connected |
|---|---|---|
| Value | number | `0.0` |

**Output**: number

> Use it as a mask: into **Factor** of Mix colour, **Position** of a Color ramp, **Roughness / Metallic / Alpha / Height** of the output, or **A/B** of Math.

Example: Noise texture (Factor) → **Exponential** (input Value) → Mix colour (Factor).

## Logarithm

Natural logarithm.

*Works in: material, object shader, camera effect*

**Inputs**

| Input | Type | When not connected |
|---|---|---|
| Value | number | `1.0` |

**Output**: number

> Use it as a mask: into **Factor** of Mix colour, **Position** of a Color ramp, **Roughness / Metallic / Alpha / Height** of the output, or **A/B** of Math.

Example: Noise texture (Factor) → **Logarithm** (input Value) → Mix colour (Factor).

## Reciprocal

1 divided by the value.

*Works in: material, object shader, camera effect*

**Inputs**

| Input | Type | When not connected |
|---|---|---|
| Value | number | `1.0` |

**Output**: number

> Use it as a mask: into **Factor** of Mix colour, **Position** of a Color ramp, **Roughness / Metallic / Alpha / Height** of the output, or **A/B** of Math.

Example: Noise texture (Factor) → **Reciprocal** (input Value) → Mix colour (Factor).

## Inverse lerp

Where the value lies between from and to (0..1 inside).

*Works in: material, object shader, camera effect*

**Inputs**

| Input | Type | When not connected |
|---|---|---|
| Value | number | `0.5` |

**Parameters** (on the node or in the Inspector)

| Parameter | Default | Range | Choices |
|---|---|---|---|
| from | 0 | -100..100 | - |
| to | 1 | -100..100 | - |

**Output**: number

> Use it as a mask: into **Factor** of Mix colour, **Position** of a Color ramp, **Roughness / Metallic / Alpha / Height** of the output, or **A/B** of Math.

Example: Noise texture (Factor) → **Inverse lerp** (input Value) → Mix colour (Factor).

## Map range

Remaps a range to another and clamps the result.

*Works in: material, object shader, camera effect*

**Inputs**

| Input | Type | When not connected |
|---|---|---|
| Value | number | `0.5` |

**Parameters** (on the node or in the Inspector)

| Parameter | Default | Range | Choices |
|---|---|---|---|
| in min | 0 | -100..100 | - |
| in max | 1 | -100..100 | - |
| out min | 0 | -100..100 | - |
| out max | 1 | -100..100 | - |

**Output**: number

> Use it as a mask: into **Factor** of Mix colour, **Position** of a Color ramp, **Roughness / Metallic / Alpha / Height** of the output, or **A/B** of Math.

Example: Noise texture (Factor) → **Map range** (input Value) → Mix colour (Factor).

## Snap

Rounds to the nearest multiple of the step.

*Works in: material, object shader, camera effect*

**Inputs**

| Input | Type | When not connected |
|---|---|---|
| Value | number | `0.5` |

**Parameters** (on the node or in the Inspector)

| Parameter | Default | Range | Choices |
|---|---|---|---|
| step | 0.1 | 0.0001..100 | - |

**Output**: number

> Use it as a mask: into **Factor** of Mix colour, **Position** of a Color ramp, **Roughness / Metallic / Alpha / Height** of the output, or **A/B** of Math.

Example: Noise texture (Factor) → **Snap** (input Value) → Mix colour (Factor).

## Arc tangent 2

Angle of the point (X, Y) in radians.

*Works in: material, object shader, camera effect*

**Inputs**

| Input | Type | When not connected |
|---|---|---|
| Y | number | `0.0` |
| X | number | `1.0` |

**Output**: number

> Use it as a mask: into **Factor** of Mix colour, **Position** of a Color ramp, **Roughness / Metallic / Alpha / Height** of the output, or **A/B** of Math.

Example: Noise texture (Factor) → **Arc tangent 2** (input Y) → Mix colour (Factor).

## Smootherstep

A smoother version of Smoothstep.

*Works in: material, object shader, camera effect*

**Inputs**

| Input | Type | When not connected |
|---|---|---|
| Value | number | `0.5` |

**Parameters** (on the node or in the Inspector)

| Parameter | Default | Range | Choices |
|---|---|---|---|
| edge 0 | 0 | -10..10 | - |
| edge 1 | 1 | -10..10 | - |

**Output**: number

> Use it as a mask: into **Factor** of Mix colour, **Position** of a Color ramp, **Roughness / Metallic / Alpha / Height** of the output, or **A/B** of Math.

Example: Noise texture (Factor) → **Smootherstep** (input Value) → Mix colour (Factor).

## Less than

1 where A < B, otherwise 0.

*Works in: material, object shader, camera effect*

**Inputs**

| Input | Type | When not connected |
|---|---|---|
| A | number | `0.0` |
| B | number | `0.5` |

**Output**: number

> Use it as a mask: into **Factor** of Mix colour, **Position** of a Color ramp, **Roughness / Metallic / Alpha / Height** of the output, or **A/B** of Math.

Example: Noise texture (Factor) → **Less than** (input A) → Mix colour (Factor).

## Average

Mean of two values.

*Works in: material, object shader, camera effect*

**Inputs**

| Input | Type | When not connected |
|---|---|---|
| A | number | `0.0` |
| B | number | `1.0` |

**Output**: number

> Use it as a mask: into **Factor** of Mix colour, **Position** of a Color ramp, **Roughness / Metallic / Alpha / Height** of the output, or **A/B** of Math.

Example: Noise texture (Factor) → **Average** (input A) → Mix colour (Factor).

## Math

One node for 26 operations on numbers (add, power, smooth min, snap, ping-pong...). Pick the operation in the list.

*Works in: material, object shader, camera effect*

**Inputs**

| Input | Type | When not connected |
|---|---|---|
| A | number | `0.5` |
| B | number | `0.5` |
| C | number | `0.5` |

**Parameters** (on the node or in the Inspector)

| Parameter | Default | Range | Choices |
|---|---|---|---|
| operation | 2 | 0..25 | 0 Add, 1 Subtract, 2 Multiply, 3 Divide, 4 Power, 5 Logarithm, 6 Square root, 7 Absolute, 8 Minimum, 9 Maximum, 10 Less than, 11 Greater than, 12 Round, 13 Floor, 14 Ceil, 15 Fraction, 16 Modulo, 17 Sine, 18 Cosine, 19 Tangent, 20 Arctan2, 21 Smooth minimum, 22 Smooth maximum, 23 Snap, 24 Ping-pong, 25 Multiply add |
| clamp | 0 | 0..1 | 0 Off, 1 Clamp 0..1 |

**Output**: number

> Use it as a mask: into **Factor** of Mix colour, **Position** of a Color ramp, **Roughness / Metallic / Alpha / Height** of the output, or **A/B** of Math.

Example: Noise texture (Factor) → **Math** (input A) → Mix colour (Factor).

## Float curve

Reshapes a value by a smooth curve through five points.

*Works in: material, object shader, camera effect*

**Inputs**

| Input | Type | When not connected |
|---|---|---|
| Value | number | `0.5` |

**Parameters** (on the node or in the Inspector)

| Parameter | Default | Range | Choices |
|---|---|---|---|
| y at 0 | 0 | -1..2 | - |
| y at 0.25 | 0.25 | -1..2 | - |
| y at 0.5 | 0.5 | -1..2 | - |
| y at 0.75 | 0.75 | -1..2 | - |
| y at 1 | 1 | -1..2 | - |

**Output**: number

> Use it as a mask: into **Factor** of Mix colour, **Position** of a Color ramp, **Roughness / Metallic / Alpha / Height** of the output, or **A/B** of Math.

Example: Noise texture (Factor) → **Float curve** (input Value) → Mix colour (Factor).

## Clamp range

Keeps a value between two inputs.

*Works in: material, object shader, camera effect*

**Inputs**

| Input | Type | When not connected |
|---|---|---|
| Value | number | `0.5` |
| Min | number | `0.0` |
| Max | number | `1.0` |

**Output**: number

> Use it as a mask: into **Factor** of Mix colour, **Position** of a Color ramp, **Roughness / Metallic / Alpha / Height** of the output, or **A/B** of Math.

Example: Noise texture (Factor) → **Clamp range** (input Value) → Mix colour (Factor).

