# NVIDIA и генерация кадров

Проверено 28 октября 2026. SDK скачаны в локальную, исключённую из Git папку
`third_party/nvidia/`. Драйверы и системные настройки не менялись.

| SDK | Локальная версия | Назначение |
|---|---|---|
| NVIDIA Streamline | 2.14.16 | DLSS SR/RR/FG, Reflex, DLSS Neural Rendering; интеграционный SDK |
| RTX Neural Shading | 1.5.0, commit `f309ae68677a67060cc6ded2d0af925a3931435f` | Slang, cooperative vectors, обучение и inference в шейдерах |

SHA256 официального x64 архива Streamline:
`32041e51db0cab709a271b72a352281b690c2abbd669a827809401a5dd23956b`.

Источники: [Streamline releases](https://github.com/NVIDIA-RTX/Streamline/releases),
[руководство DLSS FG](https://github.com/NVIDIA-RTX/Streamline/blob/main/docs/ProgrammingGuideDLSS_G.md),
[RTXNS](https://github.com/NVIDIA-RTX/RTXNS),
[VRS](https://developer.nvidia.com/vrworks/graphics/variablerateshading).

## Что действительно работает в Kvas

- Собственный пространственный апскейлер EASU/RCAS-style; это не официальный AMD FSR SDK.
- Экспериментальная optical-flow интерполяция ×2: GPU оценивает блочное движение,
  синтезирует середину двух отрендеренных кадров и затем показывает реальный кадр.
  Работает в Game/Play; simulation/input продолжают обновляться на кадрах интерфейса.
  История сбрасывается при смене размеров, большом скачке камеры, паузе или отключении.
  HUD рисуется отдельно поверх результата.
- Эта интерполяция добавляет задержку и может давать артефакты при окклюзиях,
  прозрачности, быстрых движениях и движении более 8 пикселей за реальный кадр.
  Она не гарантирует рост FPS: постпроходы и VSync также ограничивают производительность.
- Чистый PNG export/capture всегда снимает реальный кадр, без интерполяции.

## Что ещё требует отдельного бэкенда

Наличие SDK на диске **не включает DLSS**. Текущий wgpu/eframe путь не передаёт
Streamline нативные ресурсы и swapchain. Для DLSS FG нужны depth, motion vectors,
HUD-less color, согласованные camera matrices/frame tokens, Reflex и управление present.
Нужны проверка подписей NVIDIA DLL, аппаратная проверка через SDK и лицензионная
проверка redistributables перед упаковкой. Подмена названия GPU проверкой поддержки не является.

Выбор DLSS пока явно использует собственный пространственный fallback. DLSS SR/RR/FG,
Neural Rendering, аппаратные RT/path tracing, VRS и cooperative-vector inference
**не интегрированы** в Kvas. RTXNS примеры не запускались и сети на GPU не обучались.

Термин «VSA» из задания неоднозначен: не переименовываем его молча в FSR или VRS.
Ссылка/точное название нужны для отдельной реализации.

## Повторная загрузка

Запустите `pwsh -File tools/fetch_nvidia.ps1`. Скрипт загружает фиксированный официальный
архив Streamline, проверяет SHA256 и клонирует фиксированный тег RTXNS. Бинарники SDK
и их исходники не публикуются вместе с движком; лицензии находятся в самих SDK.
