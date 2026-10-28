# Vertices, UVs and smoothing

`EditableMesh` is geometry you can change from scripts and from the editor. Every method works in the object's **local space**,
marks the mesh as changed (the GPU gets a new version) and leaves the normals alone - call `recalculate_normals()` when lighting matters.

## Getting a mesh

```rust
if ctx.make_editable(entity) {            // turns primitive:cube/sphere/... into an editable mesh
    if let Some(mesh) = ctx.editable(entity) {
        mesh.twist(Vec3::Y, 30.0);
        mesh.recalculate_normals().ok();
    }
}
```

For models from files (`.glb`, `.obj`) the editor's **Geometry** window creates the editable mesh (the `EditableMesh` component).

## Vertex access

| Method | Description |
|---|---|
| `vertex_count()`, `position(i)`, `set_position(i, p)` | read and write one vertex |
| `bounds()` | `(min, max)` of the bounding box |
| `map_positions(\|i, p, n\| new_position)` | any custom deformation |
| `select_sphere(center, r)`, `select_half_space(point, normal)` | select vertices |
| `translate_vertices(&ids, delta)`, `translate`, `scale_about`, `transform(mat)` | shifts and matrices |

## Deformers

| Method | Result |
|---|---|
| `wave(direction, displace, amplitude, wavelength, phase)` | a travelling wave (flags, water) |
| `twist(axis, degrees_per_metre)` | twisting |
| `taper(axis, start, end)` | narrowing or widening along an axis |
| `bend(along, axis, degrees_per_metre)` | bending |
| `inflate(amount)` | push along the normals |
| `noise_displace(&noise, amplitude, time)` | irregularities from noise (rocks, terrain) |
| `smooth(iterations, factor)` | Laplacian smoothing |

```rust
let noise = EditableMesh::default_noise(2.0, 7);   // Perlin, frequency 2, seed 7
mesh.noise_displace(&noise, 0.15, ctx.elapsed());  // a "breathing" rock
mesh.smooth(2, 0.4);
mesh.recalculate_normals().ok();
```

## UV unwrapping

| Method | When to use |
|---|---|
| `project_planar(axis, scale)` | floors, walls, decals: `scale` is metres per texture repeat |
| `project_cylindrical(scale)` | trunks, columns, bottles |
| `project_spherical()` | balls, planets |
| `normalize_uv(margin)` | fit the UVs into 0..1 (for baking) |
| `transform_uv(scale, rotation_deg, offset)` | scale, rotate and shift the unwrap |

```rust
mesh.project_planar(Vec3::Y, 2.0);   // seen from above, 2 m per repeat
mesh.transform_uv([1.0, 1.0], 45.0, [0.0, 0.0]);
```

## Normal smoothing

| Method | Result |
|---|---|
| `recalculate_normals()` | smooth normals weighted by triangle area |
| `flat_normals()` | faceted: every triangle has its own normal |
| `smooth_normals(angle)` | smooth, but edges sharper than `angle` degrees stay hard (like Auto Smooth) |

For a low-poly style use `flat_normals()`; for props with bevels use `smooth_normals(35.0)`.

## Limits

An editable mesh is limited to 100 000 vertices and 600 000 indices. For very large meshes deform in the shader (the **Wave offset** and **Wind sway** nodes).
