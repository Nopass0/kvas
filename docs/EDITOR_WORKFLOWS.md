# Kvas: рабочие сценарии редактора

## Начало и демо

`cargo run -p kvas_editor -- games/demo` открывает базовое демо.
`cargo run -p kvas_editor -- games/demo --scene scenes/pbr_lab.scene.json --play`
показывает PBR-лабораторию с золотом, хромом, керамикой и процедурными материалами.
Панорама Studio Small 09 (Poly Haven, CC0) включена в проект; повторная загрузка —
`pwsh -File tools/fetch_demo_assets.ps1`. `make_pbr_lab` регенерирует только эту сцену/материалы.
Другие переносимые демо: `games/showcase`, `games/test_meadow`, `games/test_cloth`, `games/test_voxels`.

## Ноды: материалы, шейдеры, камера

Создайте Material (.kmat), Shader graph (.kshader) или Camera effect (.kfx) в Project.
Материал/шейдер перетащите на объект. Эффект камеры перетащите в Scene/Inspector.
Выход графа выбирается по его типу: Output / Shader output / Effect output.

Shift+A или Space открывает поиск нод; средняя кнопка двигает граф; колесо масштабирует.
Home вмещает граф; рамка выделяет несколько нод; Ctrl+C/V/D копирует/вставляет/дублирует;
Ctrl+Z/Y отменяет/повторяет; Delete удаляет выбранное. Comment и Reroute помогают организовать граф.
Мини-карта, Auto layout и выбор модели предпросмотра находятся в окне нод.
Меню добавления делится на категории (Ввод, Геометрия, Свет, Нормали, Шумы, Узоры, Цвет, Математика, Векторы) с описанием каждой ноды; стрелки и Enter выбирают.
Шумы и узоры показывают превью внутри ноды (правый клик - Show / Hide preview in node). Подробности: [нодовый редактор](nodes/index.md), [справочник](nodes/reference.md).
Флаг предпросмотра выбранной ноды показывает промежуточный выход для mesh-графов.
Для .kfx результат виден на камере сцены; отдельного промежуточного camera preview пока нет.

Все сокеты передают vec3: скаляр повторяется в RGB. Нулевые константы сохраняются точно.
Inspector number читает MeshRenderer.user[0..7]; в эффектах камеры доступны params[0..3].
Параметр `zero uses default = 0` позволяет передавать точный ноль; 1 сохраняет прежний fallback.
Screen position даёт нормализованные screen UV реального render target; UV доступен также в .kfx.

Vertex offset задаётся в мировых координатах в Shader output. Texture, Normal map,
Bump from height, Shadow, Lighting, Sky reflection и Screen position требуют fragment stage
и не могут питать Vertex offset. Редактор выдаёт понятную ошибку, вместо некорректного WGSL.
Граф проверяется на циклы, дубликаты ID, отсутствующие ссылки, неконечные значения и глубину >256.
Экспорт WGSL односторонний: произвольный текст обратно в граф не разбирается.

## Выделение, иерархия и горячие клавиши

Ctrl+клик переключает объект в выделении. Shift+клик расширяет диапазон иерархии;
в Scene Shift добавляет объект. W/E/R переключает gizmo, Ctrl во время drag меняет привязку.
Групповые gizmo, Ctrl+D и Delete работают с корнями выделения: дочерний объект не обрабатывается дважды.
Инспектор показывает активный объект. Перетаскивание группы в иерархии меняет родителей корней.
Сброс на пустую область отсоединяет их. Циклы родителей не допускаются.

F фокусирует активный объект; RMB+WASDQE — полёт; Shift ускоряет; Alt+LMB — орбита;
MMB — панорамирование. Ctrl+P запускает/останавливает Play. Ctrl+S сохраняет сцену.
Ctrl+Alt+S открывает настройки. Окна настроек, сборки, создания и новых редакторов перемещаются за заголовок.
Если открыт редактор нод/кода/изображений, глобальные команды сцены не перехватывают его ввод.

У каждого объекта доступна строка «Теги». Разделяйте значения пробелами; регистр учитывается.
В скрипте: `ctx.tagged("enemy")`, `ctx.has_tag(entity, "enemy")`, `ctx.add_tag(...)`.
SVG-иконки показывают типы объектов/компонентов; исходники находятся в crates/kvas_editor/icons.

## Геометрия, Sculpt и UV

Выберите mesh-объект → Inspector → «Редактировать геометрию / Sculpt» или меню Window.
Начало редактирования копирует текущую геометрию, запекает стек модификаторов и добавляет EditableMesh.
Импортированные material runs не встраиваются в EditableMesh: назначайте текстуры явно в MeshRenderer.
Процедурные Cloth/Grass/Water/Smoke не редактируются этим инструментом напрямую.

В Scene кликайте вершины; Ctrl/Shift добавляет. Видимые маркеры ограничены по количеству;
выбор не проверяет перекрытие поверхностями. В окне есть список XYZ и смещение выделенных вершин.
Sculpt выдавливает поверхность вдоль нормали, Ctrl вдавливает. Радиус измеряется локально;
для деталей нужна достаточно плотная сетка. После штриха нормали пересчитываются.
EditableMesh сохраняется в сцене и работает в отдельной игре. Ctrl+Z отменяет изменения сцены.

Box UV — простая проекция по доминирующей нормали, а не полноценный smart unwrap с упаковкой островов.
«Проверка UV» включает UV debug view. OBJ экспортируется в новый файл models/edited;
экспорт содержит позиции, UV, нормали, но не glTF материал/скелет.
Лимит редактирования: 100000 вершин и 600000 индексов. Subdivision/Boolean/retopology не реализованы здесь.
«Fit collider» использует текущую геометрию и повторно использует дочерний Auto Collider;
это box approximation, а не точный mesh collider.

## Texture paint и редактор изображений

TexturePaint смешивает четыре слоя на поверхности. Включите кисть, выберите слой,
радиус/силу/жёсткость; Soften сглаживает маску. Штрих и Fill отменяются Ctrl+Z/Y
или кнопками панели кисти. История масок ограничена 64 MiB и действует в текущем проекте.
Save mask / Save Scene пишет маски в assets/paint.

Window → «Изображения — слои и кисти»: прозрачный холст 512², до восьми слоёв,
видимость/opacity/порядок/имя, кисть и ластик с мягкими краями, лассо и прямоугольное выделение,
заливка выбранной области. Кисть рисует непрерывные штрихи; Ctrl+Z/Y отменяет/повторяет.
PNG + слои сохраняет flattened PNG и редактируемый .kimage рядом. Откройте .kimage для восстановления слоёв.
Укажите путь внутри assets, например textures/stone.png. Максимум 1024×1024, история до 64 MiB.
Pressure используется при получении Touch.force от платформы; аппаратный стилус не проверен.
Редактор не претендует на Photoshop: нет PSD, color management, blend modes, masks и filters.

## Анимация объектов

Добавьте TransformAnimation или откройте Window → «Анимация — таймлайн» для выбранного компонента.
Установите время, задайте Transform объекта и нажмите «Записать позу».
Ключ хранит position/rotation/scale. Перемотка показывает pose, список позволяет менять время и XYZ.
Step держит pose, Linear интерполирует линейно, Smooth использует smoothstep;
вращение всегда идёт кратчайшим quaternion-путём. Есть looping, reverse speed и playing.
Воспроизведение работает в Play и плеере. Сохраняйте сцену после записи ключей.
Динамическая физика может перезаписывать Transform: используйте статические/кинематические объекты.
Скелеты, skinning, glTF animation clips, IK, animation state machine и root motion пока отсутствуют.

## Звук

Во вкладке Sound выберите клип: показываются waveform, длительность, channels/sample rate.
Можно прослушать, настроить volume/pitch/pan, создать sound object или назначить AudioSource.
«Редактор клипа» обрезает диапазон в секундах, применяет fade in/out и peak normalization 98%.
Export пишет новый PCM16 WAV в sounds/edits, исходник не меняется. Обработка идёт на worker.
Конец 0 означает весь клип. Лимит обработки — 32 млн interleaved samples.
Это редактор одного клипа; многодорожечный timeline/микшер/реверберация пока не реализованы.

## AI: обучение, веса, применение

Window → «AI — обучение / веса»: CPU MLP 2→8→2 обучается по JSON input/target примерам.
Пустой dataset использует 25 steering примеров; данные нормализуйте в -1..1.
Настраиваются epochs и learning rate; видны progress, MSE и loss curve; есть отмена.
«Сохранить веса» создаёт JSON в assets/ai, «Загрузить веса» проверяет форму и конечность чисел.
«Применить к объекту» копирует сеть в NeuralAgent. Задайте target (имя объекта), speed,
input_range и включите enabled; сохраните сцену и запустите Play.

NeuralAgent получает нормализованные X/Z смещения к target и предсказывает X/Z движение.
Не управляет RigidBody и не обходит препятствия. Это supervised imitation/regression,
не reinforcement learning, NavMesh, behavior tree или полноценная система обучения врагов.
Веса переносимы; script API: `kvas_engine::neural::Network::predict(&features)`.
Лимит UI обучения — 200 млн оценочных операций; GPU training/inference не запускается.

## GUI на экране и в мире

Window → «GUI — интерфейс игры»: создайте Label, Button или Progress и перетащите на canvas.
В инспекторе UiElement доступны text, normalized anchor, pixel offset/size, цвета,
font size, value, visibility/interactable и signal. Anchor.z задаёт порядок отрисовки.
world_space привязывает элемент к world Transform, проецируя его на камеру.
Это billboard overlay без occlusion/depth, не настоящая 3D поверхность.

Кнопка публикует сигнал в следующий tick. В update: `for event in ctx.signals("ui.click") { ... }`.
Sender — сущность кнопки, payload `{ "clicked": true }`. Для игрового меню снимайте cursor lock.
UiElement сериализуется и отрисовывается одним shell-путём в Play и отдельной игре.
Нет flex/grid layout, UI images, text input и полного gamepad focus navigation.

## Графика и настройки

Settings содержит Appearance, Code editor, Scene, Shortcuts, General, Input,
Simulation, Sound, Build Game и Rendering. Настройки редактора сохраняются автоматически.
General/Input/Simulation/Sound/Build сохраняются в project.json кнопкой Save;
Rendering хранится в текущей сцене и сохраняется отдельно кнопкой Save Scene.
Ошибки manifest validation появляются в Console; неверный manifest не сохраняется через Settings.

Simulation задаёт физическую частоту, предел dt и число шагов catch-up; параметры общие для Play/плеера.
HDRI выбирается в Rendering или Scene Settings; поддержаны .hdr (linear), PNG/JPEG (sRGB),
rotation/intensity, фон, зеркальные отражения и усреднённый diffuse ambient.
Карта ограничивается 2048×1024. Полноценного prefiltered specular IBL / GI пока нет.

Frame generation — экспериментальный собственный optical flow ×2 в Game. Он добавляет задержку,
может давать артефакты и не гарантирует увеличение FPS. Это не NVIDIA DLSS FG.
DLSS SDK скачан, но native Streamline backend не подключён; выбор DLSS использует spatial fallback.
Реальных ray tracing/path tracing и NVIDIA Neural Rendering пока нет. Детали — docs/NVIDIA.md.

## Сборка и проверка игры

File → Build Game или `kvas_editor --build games/demo` собирает актуальные player/scripts,
пакет и exe, после чего игра должна реально отрисовать минимум 12 кадров и сохранить отчёт/PNG.
Отсутствующий camera, ошибка загрузки или рендера делают smoke ошибочным. Отчёт лежит в build/smoke-report-*.json.
Вместо прежней проверки «процесс жив несколько секунд» используется rendered-frame evidence.

Для быстрой локальной проверки без script DLL: `kvas_player --project games/demo --scene scenes/pbr_lab.scene.json --no-scripts --smoke-report tools/pbr-smoke.json`.
Для проверки скриптов нужен обычный build: пропуск scripts не является доказательством игровой логики.
`pwsh -File tools/verify.ps1` проверяет workspace, tests, executable docs и rustdoc;
`-Gpu` дополнительно запускает ограниченные native GPU tests. `cargo test -p kvas_render --lib -- --ignored`
проверяет pixels/pacing интерполяции. Тесты не заменяют ручную проверку всех UI flows.
