---
layout: home
title: Kvas
hero:
  name: Kvas
  text: Игровой 3D-движок на Rust
  tagline: Редактор в стиле Unity, нодовые материалы, скрипты на Rust с горячей перезагрузкой и общий рендерер для редактора и игры.
  image:
    src: /images/demo.png
    alt: Сцена демо-проекта Kvas
  actions:
    - theme: brand
      text: Быстрый старт
      link: /guide/getting-started
    - theme: alt
      text: Скрипты
      link: /scripting/
    - theme: alt
      text: Нодовые материалы
      link: /nodes/
features:
  - icon: 🧩
    title: Нодовый редактор
    details: 190 нод - шумы с превью, Color Ramp, карты нормалей и высот, свет, углы, стороны, свечение. Материалы, шейдеры объектов и эффекты камеры.
    link: /nodes/
  - icon: 🦀
    title: Скрипты на Rust
    details: Один файл - один скрипт. Горячая перезагрузка, поля в Инспекторе, помощники look_at, move_towards, Rng, lerp и десятки методов Ctx.
    link: /scripting/
  - icon: 🎬
    title: Один рендерер
    details: wgpu рисует и редактор, и собранную игру. PBR, тени, HDRI, небо, частицы, вода, трава, постобработка.
    link: /GUIDE
  - icon: 🛠️
    title: Редактор
    details: Иерархия, Инспектор, гизмо, Sculpt и UV, texture paint, анимация, звук, GUI, сборка игры в одну папку.
    link: /EDITOR_WORKFLOWS
  - icon: 📚
    title: Документация, которая не врёт
    details: Примеры скриптов компилируются и запускаются в тестах, справочник нод создаётся из кода, Rust API собирается rustdoc.
    link: /ENGINE_API
  - icon: ⚡
    title: Апскейл и кадры
    details: Собственный апскейлер в стиле FSR и экспериментальная интерполяция кадров. Честно о том, что делает DLSS и чего ещё нет.
    link: /NVIDIA
---
