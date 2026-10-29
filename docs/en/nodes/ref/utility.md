---
outline: [2, 3]
---

# Utility

Reroute, comments, number from the Inspector.

Generated from the node library. Socket types: **number** (grey circle), **colour** (yellow circle), **vector** (purple diamond). Any output can feed any input; when the types differ, pick a conversion on the input (right-click it). See [how nodes connect](../wiring).

## Reroute

A tiny pass-through point to tidy long links.

*Works in: material, object shader, camera effect*

**Inputs**

| Input | Type | When not connected |
|---|---|---|
| Value | number | `0.0` |

**Output**: number

> Use it as a mask: into **Factor** of Mix colour, **Position** of a Color ramp, **Roughness / Metallic / Alpha / Height** of the output, or **A/B** of Math.

Example: Noise texture (Factor) → **Reroute** (input Value) → Mix colour (Factor).

## Comment

A frame with a note. Nodes inside it move with it.

*Works in: material, object shader, camera effect*

**Output**: number

> Use it as a mask: into **Factor** of Mix colour, **Position** of a Color ramp, **Roughness / Metallic / Alpha / Height** of the output, or **A/B** of Math.

## Inspector number

A number you can change per object in the Inspector (MeshRenderer user 1..8) or from a script.

*Works in: material, object shader, camera effect*

**Parameters** (on the node or in the Inspector)

| Parameter | Default | Range | Choices |
|---|---|---|---|
| slot 0-7 | 0 | 0..7 | - |
| default | 1 | -1000..1000 | - |
| zero uses default | 1 | 0..1 | - |

**Output**: number

> Use it as a mask: into **Factor** of Mix colour, **Position** of a Color ramp, **Roughness / Metallic / Alpha / Height** of the output, or **A/B** of Math.

## Time scaled

Time multiplied by a speed.

*Works in: material, object shader, camera effect*

**Parameters** (on the node or in the Inspector)

| Parameter | Default | Range | Choices |
|---|---|---|---|
| speed | 1 | -50..50 | - |

**Output**: number

> Use it as a mask: into **Factor** of Mix colour, **Position** of a Color ramp, **Roughness / Metallic / Alpha / Height** of the output, or **A/B** of Math.

## Pulse

A smooth 0..1 wave over time.

*Works in: material, object shader, camera effect*

**Parameters** (on the node or in the Inspector)

| Parameter | Default | Range | Choices |
|---|---|---|---|
| hertz | 1 | 0..20 | - |

**Output**: number

> Use it as a mask: into **Factor** of Mix colour, **Position** of a Color ramp, **Roughness / Metallic / Alpha / Height** of the output, or **A/B** of Math.

