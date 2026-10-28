---
layout: home
title: Kvas
hero:
  name: Kvas
  text: A 3D game engine in Rust
  tagline: A Unity-style editor, node-based materials, hot-reloaded Rust scripts and one renderer for the editor and the shipped game.
  image:
    src: /images/demo.png
    alt: A scene from the Kvas demo project
  actions:
    - theme: brand
      text: Quick start
      link: /en/guide/getting-started
    - theme: alt
      text: Scripting
      link: /en/scripting/
    - theme: alt
      text: Materials and nodes
      link: /en/nodes/
features:
  - icon: 🧩
    title: Node editor
    details: 190 nodes - noise with previews, Color Ramp, normal and height maps, light, angles, faces, glow. Materials, object shaders and camera effects.
    link: /en/nodes/
  - icon: 🎨
    title: 80 built-in materials
    details: Realistic and stylized procedural materials in every new project - stone, metal, wood, fabric, lava, toon, galaxy, holographic foil.
    link: /en/nodes/materials
  - icon: 🦀
    title: Rust scripts, made easy
    details: One `script!` block declares variables for the Inspector and the callbacks. Helpers for movement, hierarchy, tags, shared variables, vertices and UVs.
    link: /en/scripting/easy
  - icon: 🛠️
    title: Mesh tools
    details: Deform vertices from scripts - wave, twist, bend, noise - plus planar, cylindrical and spherical UV projection and crease-angle smoothing.
    link: /en/scripting/mesh
  - icon: 🔊
    title: Directional sound
    details: Spatial audio with a cone angle - a loudspeaker that is quieter behind and to the sides - and gizmos that show it in the scene.
    link: /en/scripting/
  - icon: 📚
    title: Documentation that stays true
    details: Script examples are compiled and run by the test-suite, the node reference and the material catalogue are generated from the code.
    link: /en/scripting/cookbook
---
