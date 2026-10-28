---
outline: [2, 2]
---

# Справочник нод

Страница создаётся автоматически из библиотеки нод (`cargo run -p kvas_engine --example gen_node_docs`). Всего нод: **190**.

Буквы показывают, где ноду можно использовать: **M** - материал (`.kmat`), **S** - шейдер объекта (`.kshader`), **E** - эффект камеры (`.kfx`).

Параметры показаны как `имя (мин..макс)`. Подсказки - те же, что видны в редакторе.

## Ввод

Константы, текстуры, время, UV.

| Нода | Где | Входы | Параметры | Описание |
|---|---|---|---|---|
| **Color** | MSE | - | цвет | A constant colour. |
| **Value** | MSE | - | value (-100..100) | A constant number. |
| **Texture** | MS | UV | slot (0..2), tiling (0.01..100) | Samples one of the material images (slot 0 albedo, 1 normal, 2 ORM). |
| **Time** | MSE | - | - | Seconds since the game started. |
| **Object colour** | MS | - | - | Цвет, заданный в MeshRenderer объекта. |
| **Triplanar texture** | MS | - | slot (0..2), scale (0.01..50), sharpness (1..32) | Проецирует картинку материала по трём осям мира, без растяжения на склонах. |

## Геометрия

Позиция, нормаль, высота, маски по направлению, смещение вершин.

| Нода | Где | Входы | Параметры | Описание |
|---|---|---|---|---|
| **UV** | MSE | - | - | Texture coordinates of the surface (0..1). |
| **World position** | MS | - | - | Position of the pixel in the world. |
| **Normal** | MS | - | - | Direction the surface faces. |
| **Height** | MS | - | - | World height (Y) of the pixel. |
| **Slope mask** | MS | - | from (0..1), to (0..1) | 1 на ровной земле, 0 на крутых склонах (между двумя порогами). |
| **Camera distance** | MS | - | range (1..1000) | 0 у камеры, 1 на выбранной дальности (туман, детализация). |
| **View direction** | MS | - | - | Единичный вектор от поверхности к камере. |
| **Reflection** | MS | - | - | Направление взгляда, отражённое нормалью поверхности. |
| **Camera position** | MS | - | - | Позиция камеры в мире. |
| **Object position** | MS | - | - | Позиция центра объекта в мире. |
| **Object random** | MS | - | - | Случайное число, разное у каждого объекта, но постоянное во времени. |
| **Screen position** | MS | - | - | Позиция пикселя на экране (0..1). |
| **Direction mask** | MS | - | x (-1..1), y (-1..1), z (-1..1), threshold (-1..1), softness (0.001..2) | Белое там, где поверхность смотрит в выбранном направлении (x, y, z). Верх = (0,1,0). |
| **Surface sides** | MS | - | - | Красный - верхние грани, зелёный - боковые, синий - нижние. Разделите нодами Red / Green / Blue. |
| **Triplanar weights** | MS | - | sharpness (1..32) | С какой силой действует проекция по каждой мировой оси (X, Y, Z). |
| **Local position** | MS | - | - | Позиция относительно центра объекта. |
| **Object distance** | MS | - | range (0.1..500) | 0 в центре объекта, 1 на выбранном расстоянии. |
| **World gradient** | MS | - | x (-1..1), y (-1..1), z (-1..1), from (-1000..1000), to (-1000..1000) | Градиент 0..1 вдоль мирового направления между двумя значениями (туман по высоте, снеговая линия). |
| **Planar UV** | MS | - | axis (0..2), scale (0.01..100) | Текстурные координаты, спроецированные из мирового пространства вдоль оси. |
| **Normal UV** | MS | - | - | Сферические координаты из нормали (для небесного маппинга). |
| **Wave offset** | MS | - | height (0..5), frequency (0..30), speed (-10..10) | Смещение вершин для волн. Подключается в Shader output > Vertex offset. |
| **Wind sway** | MS | - | strength (0..3), speed (0..20), scale (0..10) | Смещение вершин, раскачивающее объект ветром сильнее наверху. В Vertex offset. |

## Свет

Солнце, тени, блики, контровой свет, свечение (Emission).

| Нода | Где | Входы | Параметры | Описание |
|---|---|---|---|---|
| **Fresnel** | MS | Power | - | Brighter at grazing angles (rim light, glass, water edges). |
| **Sun light** | MS | - | - | Насколько поверхность обращена к солнцу (0..1), без теней. |
| **Sky reflection** | MS | - | - | What the sky looks like in the mirror direction. |
| **Sun direction** | MS | - | - | Единичный вектор в сторону солнца. |
| **Sun colour** | MS | - | - | Цвет и сила солнца. |
| **Ambient light** | MS | - | - | Цвет постоянного рассеянного света сцены. |
| **Shadow** | MS | - | - | 1 in light, 0 in the sun shadow. |
| **Lighting** | MS | Base | roughness (0..1) | Applies scene lighting (sun, shadows, lamps) to a colour. |
| **Light angle** | MS | - | - | Скалярное произведение нормали и направления на солнце: 1 - к солнцу, -1 - от него. |
| **Half lambert** | MS | - | power (0.1..4) | Мягкая освещённость солнцем (0..1) без жёсткой тёмной стороны. |
| **Specular highlight** | MS | - | shininess (1..512), intensity (0..10) | Блик солнца по Блинну-Фонгу. Чем больше блеск, тем меньше и резче пятно. |
| **Rim light** | MS | - | цвет, power (0.1..12), intensity (0..20) | Цветное свечение на краях, отвёрнутых от камеры. Подключайте в Emission. |
| **Backlight** | MS | - | sharpness (1..64) | Ярко, когда камера смотрит на солнце сквозь поверхность (листья, ткань, лёд). |
| **Facing ratio** | MS | - | - | 1 там, где поверхность смотрит на камеру, 0 на силуэте. |
| **Sun with shadow** | MS | - | - | Освещённость солнцем (0..1) с учётом теней. |
| **Sun lighting** | MS | - | - | Цвет солнца на освещённость с тенями. Добавьте Sky light для полного диффузного света. |
| **Sky light** | MS | - | - | Рассеянный свет неба в направлении нормали. |
| **Glow** | MSE | - | цвет, intensity (0..50) | Постоянный цвет, умноженный на яркость. В Emission - чтобы объект светился. |
| **Pulsing glow** | MSE | - | цвет, intensity (0..50), speed (0..20) | Свечение, пульсирующее во времени. |

## Нормали

Карты нормалей, бамп из высоты, смешивание нормалей.

| Нода | Где | Входы | Параметры | Описание |
|---|---|---|---|---|
| **Normal map** | MS | Map, UV | strength (0..4) | Turns a normal-map colour into a surface direction. Connect the result to Output > Normal. |
| **Bump from height** | MS | Height | strength (0..10) | Makes bumps from a height value. Connect the result to Output > Normal. |
| **Normal strength** | MS | Normal | strength (0..4) | Смешивает нормаль поверхности и подключённую нормаль. |
| **Normal blend** | MS | Base, Detail | - | Накладывает детальную нормаль на базовую. |
| **Normal to colour** | MS | Normal | - | Показывает нормаль как цвет (для превью и отладки). |
| **Noise bump** | MS | Coords | scale (0.05..100), strength (0..10) | Бугры по фрактальному шуму. Подключайте в Output > Normal. |
| **Height to normal** | MS | UV | slot (0..2), tiling (0.01..100), strength (0..32) | Строит цвет карты нормалей из высоты в слоте текстуры. Подключайте в Normal map. |

## Шумы

Фрактальные, Перлин, Вороной, трещины, облака, грязь. По умолчанию показывают превью.

| Нода | Где | Входы | Параметры | Описание |
|---|---|---|---|---|
| **Noise** | MSE | Coords | scale (0.01..100) | Простой гладкий шум (2D, использует X и Z координат). |
| **Voronoi** | MSE | Coords | scale (0.01..100) | Cell pattern: distance to the nearest random point. |
| **Cell colour** | MSE | Coords | scale (0.01..100) | Случайный цвет для каждой ячейки Вороного. |
| **Noise 3D** | MS | Coords | scale (0.01..100) | Гладкий 3D-шум без растяжения на любых поверхностях. |
| **Ridged noise** | MS | Coords | scale (0.01..100) | Шум с острыми гребнями (горы, прожилки). |
| **Turbulence** | MS | Coords | scale (0.01..100) | Клубящийся шум со складками (дым, огонь). |
| **White noise** | MSE | Coords | cells (1..1000) | Случайный серый на каждую ячейку без сглаживания (помехи). |
| **Sparkle** | MSE | Coords | cells (4..1000), density (0..1) | Редкие яркие точки (звёзды, блёстки). |
| **Fractal noise** | MSE | Coords | scale (0.01..100), octaves (1..8), lacunarity (1.2..4), gain (0.1..0.9), type (0..2) | Слоистый шум с настройкой октав, лакунарности и усиления. Типы: fBm, ridged, billow. |
| **Perlin noise** | MSE | Coords | scale (0.01..100) | Гладкий градиентный шум (0..1). |
| **Voronoi cells** | MSE | Coords | scale (0.01..100), randomness (0..1), output (0..3) | 3D-ячеистый шум. Выходы: расстояние, края ячеек, случайный id, второе расстояние. |
| **Cracks** | MSE | Coords | scale (0.01..100), width (0.005..0.5) | Тонкие трещины по границам ячеек (сухая земля, лёд, разбитый камень). |
| **Warped noise** | MSE | Coords | scale (0.01..100), warp (0..8) | Шум, искривлённый другим шумом - завитки и органичные формы. |
| **Clouds** | MSE | Coords | scale (0.01..100), coverage (0..1), softness (0.01..1) | Мягкие облакообразные пятна. Coverage задаёт заполненность. |
| **Grunge** | MSE | Coords | scale (0.01..100), contrast (0.1..8) | Маска грязи и износа. Умножайте на цвет или используйте для шероховатости. |
| **Scratches** | MSE | Coords | density (2..400), angle (-3.15..3.15) | Тонкие вытянутые царапины под углом. |

## Узоры

Шашки, кирпичи, соты, доски, плетение, градиенты и фигуры.

| Нода | Где | Входы | Параметры | Описание |
|---|---|---|---|---|
| **Checker** | MSE | Coords | scale (1..200) | Чёрно-белая шахматная доска. |
| **Stripes** | MSE | Coords | scale (1..200), sharpness (0..1) | Параллельные полосы вдоль X. |
| **Bricks** | MSE | Coords | scale (1..100), mortar (0..0.3) | Кирпичная кладка со швами. |
| **Dots** | MSE | Coords | scale (1..200), size (0.01..0.5) | Сетка круглых точек. |
| **Caustics** | MS | Coords | scale (0.05..50), speed (0..10) | Moving light patterns like under water. |
| **Hexagons** | MSE | Coords | scale (0.5..100), edge width (0..0.5) | Honeycomb pattern edges. |
| **Wave rings** | MSE | Coords | frequency (1..200), speed (-20..20) | Анимированные кольца, расходящиеся от центра. |
| **Linear gradient** | MSE | Coords | - | 0..1 слева направо (X координат). |
| **Radial gradient** | MSE | Coords | - | 1 в центре, 0 на краю. |
| **Angular gradient** | MSE | Coords | - | 0..1 по кругу вокруг центра, как стрелка часов. |
| **Circle** | MSE | Coords | radius (0..1), softness (0..0.5) | Мягкий круг. |
| **Box** | MSE | Coords | width (0..1), height (0..1), softness (0..0.5) | Мягкий прямоугольник. |
| **Wood rings** | MS | Coords | scale (0.1..50), wobble (0..2) | Tree-ring pattern for wood. |
| **Marble veins** | MS | Coords | scale (0.05..20), warp (0..20) | Wavy veins for marble. |
| **Grid lines** | MSE | Coords | cells (1..200), line width (0.001..0.5) | Сетка тонких линий (белые линии на чёрном). |
| **Planks** | MSE | Coords | rows (1..100), plank length (0.2..40), gap (0..0.3) | Доски пола со случайными оттенками и зазорами. |
| **Weave** | MSE | Coords | threads (1..200) | Тканое плетение нитей. |

## Цвет

Color ramp, режимы наложения, HSV, цветокоррекция.

| Нода | Где | Входы | Параметры | Описание |
|---|---|---|---|---|
| **Hue shift** | MSE | Colour | turns (-1..1) | Поворачивает цвета по цветовому кругу. |
| **Saturation** | MSE | Colour | amount (0..3) | 0 - серый, 1 - без изменений, больше 1 - ярче. |
| **Contrast** | MSE | Colour | amount (0..3) | Отодвигает значения от среднего серого. |
| **Brightness** | MSE | Colour | scale (0..10) | Умножает цвет. |
| **Gradient** | MSE | Position, From, To | - | Смешивает два цвета по позиции 0..1. |
| **To grey** | MSE | Colour | - | Переводит цвет в его яркость. |
| **Overlay** | MSE | Base, Blend | - | Наложение Overlay: тёмное умножается, светлое осветляется. |
| **Screen blend** | MSE | Base, Blend | - | Наложение Screen: всегда осветляет. |
| **Colourize** | MSE | Value, Tint | - | Окрашивает яркость цвета другим цветом. |
| **Ramp 3 colours** | MSE | Position, Low, Middle, High | - | Maps 0..1 to a three-colour gradient. |
| **Gamma** | MSE | Colour | gamma (0.1..5) | Гамма-коррекция. |
| **Exposure** | MSE | Colour | stops (-8..8) | Осветляет или затемняет в ступенях экспозиции. |
| **Levels** | MSE | Colour | black (0..1), white (0..2) | Задаёт чёрную и белую точки. |
| **Threshold** | MSE | Colour | level (0..1) | Чёрно-белое по порогу яркости. |
| **Tone map** | MSE | Colour | - | Кинематографичное (ACES) тональное отображение HDR-цвета. |
| **Color ramp** | MSE | Position, Stop 1, Stop 2, Stop 3, Stop 4 | position 1 (0..1), position 2 (0..1), position 3 (0..1), position 4 (0..1) | Переводит 0..1 в градиент из четырёх цветовых точек в заданных позициях. |
| **Blend colours** | MSE | A, B, Factor | mode (0..12) | Режимы наложения как в Photoshop: 0 mix, 1 multiply, 2 screen, 3 overlay, 4 add, 5 subtract, 6 difference, 7 darken, 8 lighten, 9 soft light, 10 dodge, 11 burn, 12 hard light. |
| **HSV adjust** | MSE | Colour | hue shift (-0.5..0.5), saturation (0..3), value (0..3) | Сдвигает оттенок и масштабирует насыщенность и яркость. |
| **HSV to RGB** | MSE | Hue, Saturation, Value | - | Собирает цвет из оттенка, насыщенности и яркости. |
| **RGB to HSV** | MSE | Colour | - | Раскладывает цвет на оттенок (R), насыщенность (G), яркость (B). |
| **Lift gamma gain** | MSE | Colour | lift (-0.5..0.5), gamma (0.1..4), gain (0..4) | Классическая трёхточечная цветокоррекция. |
| **Colour temperature** | MSE | Colour | warmth (-1..1) | Тёплый (оранжевый) или холодный (синий) оттенок. |
| **Colour key** | MSE | Colour | цвет, tolerance (0..1.75), softness (0..1) | Белое там, где цвет близок к выбранному. |
| **S-curve** | MSE | Value | amount (0.05..0.95) | Контрастная кривая, сохраняющая чёрное и белое. |
| **Bias curve** | MSE | Value | bias (0.01..0.99) | Сдвигает средние тона к тёмному или светлому. |

## Математика

Арифметика, кривые, сравнения, переходы.

| Нода | Где | Входы | Параметры | Описание |
|---|---|---|---|---|
| **Add** | MSE | A, B | - | A + B. |
| **Subtract** | MSE | A, B | - | A - B. |
| **Multiply** | MSE | A, B | - | A * B (окрашивание, маски). |
| **Divide** | MSE | A, B | - | A / B (безопасно: без деления на ноль). |
| **Mix** | MSE | A, B, Factor | - | Blends A and B by Factor (0 = A, 1 = B). |
| **Power** | MSE | Value, Exponent | - | Возведение в степень (кривые, заострение масок). |
| **Clamp** | MSE | Value | min (-100..100), max (-100..100) | Ограничивает значение диапазоном. |
| **One minus** | MSE | Value | - | 1 - значение (инверсия маски). |
| **Sine** | MSE | Value | frequency (0..100) | Синусоида, приведённая к 0..1. |
| **Smoothstep** | MSE | Value | edge 0 (-10..10), edge 1 (-10..10) | Плавный переход 0..1 между двумя границами. |
| **Abs** | MSE | Value | - | Модуль. |
| **Min** | MSE | A, B | - | Меньшее из двух значений. |
| **Max** | MSE | A, B | - | Большее из двух значений. |
| **Fract** | MSE | Value | - | Дробная часть (повторяется 0..1). |
| **Step** | MSE | Value | edge (0..1) | 0 ниже границы, 1 выше. |
| **Remap** | MSE | Value | in min (-10..10), in max (-10..10), out min (-10..10), out max (-10..10) | Переводит диапазон в другой (без ограничения). |
| **Negate** | MSE | Value | - | Меняет знак. |
| **Square root** | MSE | Value | - | Квадратный корень. |
| **Floor** | MSE | Value | - | Округляет вниз. |
| **Ceil** | MSE | Value | - | Округляет вверх. |
| **Round** | MSE | Value | - | Округляет до ближайшего целого. |
| **Sign** | MSE | Value | - | -1, 0 или 1. |
| **Saturate** | MSE | Value | - | Ограничивает диапазоном 0..1. |
| **Modulo** | MSE | A, B | - | Остаток от A / B (повтор). |
| **Multiply add** | MSE | A, B, C | - | A * B + C в одной ноде. |
| **Cosine** | MSE | Value | frequency (0..100) | Косинусоида, приведённая к 0..1. |
| **Triangle wave** | MSE | Value | frequency (0..100) | Зигзагообразная волна 0..1..0. |
| **Sawtooth** | MSE | Value | frequency (0..100) | Нарастающая линия с резким возвратом в 0. |
| **Square wave** | MSE | Value | frequency (0..100) | Чередует 0 и 1. |
| **Posterize** | MSE | Value | steps (2..64) | Уменьшает число уровней значений. |
| **Greater than** | MSE | A, B | - | 1 где A > B, иначе 0. |
| **Branch** | MSE | A, B, If A > B, Otherwise | - | If A > B returns the third input, otherwise the fourth. |
| **Height blend** | MSE | Height A, Height B, Blend | sharpness (0.001..2) | Маска смешивания, где побеждает более высокая из двух высот (камень сквозь снег, мох в трещинах). |
| **Exponential** | MSE | Value | - | Экспонента. |
| **Logarithm** | MSE | Value | - | Натуральный логарифм. |
| **Reciprocal** | MSE | Value | - | 1, делённая на значение. |
| **Inverse lerp** | MSE | Value | from (-100..100), to (-100..100) | Где лежит значение между from и to (0..1 внутри). |
| **Map range** | MSE | Value | in min (-100..100), in max (-100..100), out min (-100..100), out max (-100..100) | Переводит диапазон в другой с ограничением результата. |
| **Snap** | MSE | Value | step (0.0001..100) | Округляет до ближайшего кратного шагу. |
| **Arc tangent 2** | MSE | Y, X | - | Угол точки (X, Y) в радианах. |
| **Smootherstep** | MSE | Value | edge 0 (-10..10), edge 1 (-10..10) | Более гладкая версия Smoothstep. |
| **Less than** | MSE | A, B | - | 1 где A < B, иначе 0. |
| **Average** | MSE | A, B | - | Среднее двух значений. |

## Векторы

Компоненты, скалярное и векторное произведение, повороты, координаты.

| Нода | Где | Входы | Параметры | Описание |
|---|---|---|---|---|
| **Red** | MSE | Vector | - | Копирует красный канал во все каналы. |
| **Green** | MSE | Vector | - | Копирует зелёный канал во все каналы. |
| **Blue** | MSE | Vector | - | Копирует синий канал во все каналы. |
| **Combine** | MSE | R, G, B | - | Builds a vector from the red channels of three inputs. |
| **Length** | MSE | Vector | - | Длина вектора. |
| **Distance** | MSE | A, B | - | Расстояние между двумя точками. |
| **Dot** | MSE | A, B | - | Скалярное произведение (насколько совпадают два направления). |
| **Cross** | MSE | A, B | - | Векторное произведение (направление, перпендикулярное обоим). |
| **Normalize** | MSE | Vector | - | Приводит вектор к длине 1. |
| **Reflect** | MSE | Incident, Normal | - | Отражает направление относительно нормали. |
| **Rotate coords** | MSE | Coords | angle (-6.3..6.3) | Поворачивает текстурные координаты вокруг центра. |
| **Scale offset coords** | MSE | Coords | scale x (0.01..100), scale y (0.01..100), offset x (-100..100), offset y (-100..100) | Масштабирует и сдвигает текстурные координаты. |
| **Scroll coords** | MSE | Coords | speed x (-10..10), speed y (-10..10) | Двигает координаты во времени (вода, конвейер). |
| **Polar coords** | MSE | Coords | - | Переводит координаты в угол и радиус. |
| **Mirror tile** | MSE | Coords | tiles (0.1..50) | Повторяет координаты зеркально, швы пропадают. |
| **Flipbook** | MSE | Coords | columns (1..32), rows (1..32), frames per second (0..60) | Plays a grid of frames (sprite sheet) as animation. |
| **Distort coords** | MSE | Coords, Amount | scale (0.1..100) | Wobbles coordinates with noise (heat haze, water). |
| **Rotate around axis** | MSE | Vector, Axis | angle (-6.3..6.3) | Поворачивает вектор вокруг оси на угол (радианы). |
| **Refract** | MSE | Incident, Normal | index ratio (0.1..2) | Преломляет направление на поверхности (стекло, вода). |
| **Twirl coords** | MSE | Coords | amount (-20..20) | Закручивает координаты вокруг центра. |

## Утилиты

Reroute, комментарии, число из Инспектора.

| Нода | Где | Входы | Параметры | Описание |
|---|---|---|---|---|
| **Reroute** | MSE | Value | - | A tiny pass-through point to tidy long links. |
| **Comment** | MSE | - | - | A frame with a note. Nodes inside it move with it. |
| **Inspector number** | MSE | - | slot 0-7 (0..7), default (-1000..1000), zero uses default (0..1) | A number you can change per object in the Inspector (MeshRenderer user 1..8) or from a script. |
| **Time scaled** | MSE | - | speed (-50..50) | Time multiplied by a speed. |
| **Pulse** | MSE | - | hertz (0..20) | A smooth 0..1 wave over time. |

## Камера

Только в графах эффектов камеры (.kfx).

| Нода | Где | Входы | Параметры | Описание |
|---|---|---|---|---|
| **Screen color** | E | Coords | - | Colour of the camera image at the given coordinates (blur, distortion). |
| **Pixel colour** | E | - | - | The current pixel of the camera image. |
| **Pixel size** | E | - | - | Размер одного пикселя экрана в единицах 0..1. |
| **Aspect ratio** | E | - | - | Ширина экрана, делённая на высоту. |
| **Blur** | E | Coords | radius px (0..30) | Cheap 5-tap blur of the camera image. |
| **Chromatic split** | E | Coords | amount px (0..30) | Shifts the red and blue channels apart. |
| **Edge detect** | E | Coords | strength (0..20) | White where the picture changes sharply. |
| **Pixelate** | E | Coords | cells across (8..1000) | Blocky retro look. |
| **Vignette** | E | Coords | strength (0..2) | Darkens the screen edges. |
| **Scanlines** | E | Coords | lines (10..2000) | CRT-like horizontal lines. |

## Выходные ноды

| Граф | Нода | Входы |
|---|---|---|
| Материал | **Output** | Base color, Roughness, Metallic, Emission, Alpha, Occlusion, Normal, Height |
| Шейдер объекта | **Shader output** | Color, Alpha, Emission, Light amount, Roughness, Vertex offset |
| Эффект камеры | **Effect output** | Color |
