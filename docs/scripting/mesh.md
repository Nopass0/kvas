# Вершины, UV и сглаживание

`EditableMesh` - сетка, которую можно менять из скриптов и редактора. Все методы работают в **локальных координатах** объекта,
сами отмечают сетку изменённой (GPU получит новую версию) и не трогают нормали - вызовите `recalculate_normals()`, когда нужно освещение.

## Получить сетку

```rust
if ctx.make_editable(entity) {            // превращает primitive:cube/sphere/... в редактируемую
    if let Some(mesh) = ctx.editable(entity) {
        mesh.twist(Vec3::Y, 30.0);
        mesh.recalculate_normals().ok();
    }
}
```

Для моделей из файлов (`.glb`, `.obj`) редактируемую сетку создаёт окно «Геометрия» редактора (компонент `EditableMesh`).

## Доступ к вершинам

| Метод | Описание |
|---|---|
| `vertex_count()`, `position(i)`, `set_position(i, p)` | чтение и запись одной вершины |
| `bounds()` | `(min, max)` ограничивающего бокса |
| `map_positions(\|i, p, n\| новая_позиция)` | любая своя деформация |
| `select_sphere(center, r)`, `select_half_space(point, normal)` | выбор вершин |
| `translate_vertices(&ids, delta)`, `translate`, `scale_about`, `transform(mat)` | сдвиги и матрицы |

## Деформаторы

| Метод | Что получится |
|---|---|
| `wave(direction, displace, amplitude, wavelength, phase)` | бегущая волна (флаг, вода) |
| `twist(axis, degrees_per_metre)` | скручивание |
| `taper(axis, start, end)` | сужение или расширение вдоль оси |
| `bend(along, axis, degrees_per_metre)` | изгиб |
| `inflate(amount)` | раздувание вдоль нормалей |
| `noise_displace(&noise, amplitude, time)` | неровности по шуму (камни, рельеф) |
| `smooth(iterations, factor)` | лапласово сглаживание |

```rust
let noise = EditableMesh::default_noise(2.0, 7);   // Perlin, частота 2, seed 7
mesh.noise_displace(&noise, 0.15, ctx.elapsed());  // «дышащий» камень
mesh.smooth(2, 0.4);
mesh.recalculate_normals().ok();
```

## UV-развёртка

| Метод | Когда применять |
|---|---|
| `project_planar(axis, scale)` | пол, стены, декали: `scale` - метров на повтор текстуры |
| `project_cylindrical(scale)` | стволы, колонны, бутылки |
| `project_spherical()` | шары, планеты |
| `normalize_uv(margin)` | вписать UV в квадрат 0..1 (для запекания) |
| `transform_uv(scale, rotation_deg, offset)` | масштаб, поворот и сдвиг развёртки |

```rust
mesh.project_planar(Vec3::Y, 2.0);   // вид сверху, 2 м на повтор
mesh.transform_uv([1.0, 1.0], 45.0, [0.0, 0.0]);
```

## Сглаживание нормалей

| Метод | Результат |
|---|---|
| `recalculate_normals()` | сглаженные нормали по площади треугольников |
| `flat_normals()` | граненый вид: у каждого треугольника своя нормаль |
| `smooth_normals(angle)` | гладко, но рёбра острее `angle` градусов остаются жёсткими (как Auto Smooth) |

Типичная связка для low-poly стиля: `flat_normals()`; для предметов с фасками - `smooth_normals(35.0)`.

## Лимиты

Редактируемая сетка ограничена 100 000 вершин и 600 000 индексов. Для очень больших сеток деформируйте шейдером (нода **Wave offset**, **Wind sway**).
