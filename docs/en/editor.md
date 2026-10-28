# The editor

A tour of the Kvas editor. (The detailed Russian workflows are in [EDITOR_WORKFLOWS](/EDITOR_WORKFLOWS).)

## Layout

| Panel | What it does |
|---|---|
| **Scene** | the 3D view with gizmos; Lit, Albedo, Normals, Depth, UV, Lighting, Clay, Material and Creases view modes |
| **Hierarchy** | the object tree; drag a row onto another to reparent |
| **Inspector** | components as cards with icons; tags as colour chips; add a component with **Add Component** |
| **Project** | folder tree and a thumbnail grid with 3D previews of models, shaders and materials |
| **Console** | script logs: `ctx.warn` yellow, `ctx.error` red |

## Scene icons

Objects without a mesh get a clickable badge **while you edit** (hidden in Play): lights (sun / bulb), cameras, audio sources,
particle emitters, cloth and water. Click a badge to select the object. Selecting an **audio source** also draws its audible radii and
the sound cone, so a directional loudspeaker is easy to aim.

## Tags

Tags are chips at the top of the Inspector: type a word and press Enter or Space to add it, click the cross to remove it.
From a script: `ctx.tagged("enemy")`, `ctx.has_tag(entity, "enemy")`, `ctx.tagged_within("enemy", pos, 10.0)`.

## Saving

**Ctrl+S** saves the scene; node graphs and shader files save themselves a moment after every edit.
Changes made while the game is playing are discarded when you press Stop.

## Materials, shaders, scripts

- **Materials**: `assets/materials/library/` has 326 ready materials - open them in the [node editor](/en/nodes/) or drag one onto an object.
- **Shaders**: `assets/shaders/library/` has object and camera shaders (toon, hologram, fire, ice, dissolve, wind sway, outline...).
- **Scripts**: `scripts/src/<folder>/` has starter scripts; **+ New script** creates your own from a template.

## Geometry and UV

The **Geometry** window edits an `EditableMesh`: vertex selection, sculpt brush, normal recalculation, box UV and a UV check view.
The same operations (and more - wave, twist, taper, bend, planar / cylindrical / spherical UV, crease-angle smoothing) are available from scripts: see [vertices, UVs and smoothing](/en/scripting/mesh).

## Languages

The interface is available in English and Russian (Settings → Language).
