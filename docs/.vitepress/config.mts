import { defineConfig } from 'vitepress'

// GitHub Pages serves the project at https://nopass0.github.io/kvas/
const base = process.env.DOCS_BASE ?? '/kvas/'

const search = {
  provider: 'local' as const,
  options: {
    locales: {
      root: {
        translations: {
          button: { buttonText: 'Поиск', buttonAriaLabel: 'Поиск' },
          modal: {
            noResultsText: 'Ничего не найдено',
            resetButtonTitle: 'Сбросить',
            footer: { selectText: 'выбрать', navigateText: 'перейти', closeText: 'закрыть' },
          },
        },
      },
    },
  },
}

const ruSidebar = [
  {
    text: 'Начало',
    items: [
      { text: 'Быстрый старт', link: '/guide/getting-started' },
      { text: 'Рабочие сценарии редактора', link: '/EDITOR_WORKFLOWS' },
    ],
  },
  {
    text: 'Скрипты (Rust)',
    items: [
      { text: 'Обзор и шпаргалка', link: '/scripting/' },
      { text: 'Писать скрипты легко', link: '/scripting/easy' },
      { text: 'Вершины, UV, сглаживание', link: '/scripting/mesh' },
      { text: 'Рецепты (cookbook)', link: '/scripting/cookbook' },
      { text: 'Архитектура: скрипты по объектам, сигналы, сборка', link: '/scripting/architecture' },
      { text: 'Игровые системы: UI, сохранения, камеры, звук, декали', link: '/scripting/game' },
      { text: 'Rust API: руководство', link: '/ENGINE_API' },
    ],
  },
  {
    text: 'Материалы и ноды',
    items: [
      { text: 'Как работать с нодами', link: '/nodes/' },
      { text: 'Каталог материалов', link: '/nodes/materials' },
      { text: 'Как соединяются ноды', link: '/nodes/wiring' },
      { text: 'Справочник нод (таблица)', link: '/nodes/reference' },
      { text: 'Каталог шейдеров и скриптов', link: '/library' },
    ],
  },
  {
    text: 'Ноды подробно',
    collapsed: true,
    items: [
      { text: 'Ввод', link: '/nodes/ref/input' },
      { text: 'Геометрия', link: '/nodes/ref/geometry' },
      { text: 'Свет', link: '/nodes/ref/light' },
      { text: 'Нормали', link: '/nodes/ref/normal' },
      { text: 'Шумы', link: '/nodes/ref/noise' },
      { text: 'Узоры', link: '/nodes/ref/procedural' },
      { text: 'Цвет', link: '/nodes/ref/colour' },
      { text: 'Математика', link: '/nodes/ref/math' },
      { text: 'Векторы', link: '/nodes/ref/vector' },
      { text: 'Утилиты', link: '/nodes/ref/utility' },
      { text: 'Камера', link: '/nodes/ref/effect' },
    ],
  },
  {
    text: 'Руководства',
    items: [
      { text: 'Большое руководство с примерами', link: '/GUIDE' },
      { text: 'Игра LAST SERVICE', link: '/guide/last-service' },
      { text: 'NVIDIA, DLSS и генерация кадров', link: '/NVIDIA' },
    ],
  },
]

const enSidebar = [
  {
    text: 'Getting started',
    items: [
      { text: 'Quick start', link: '/en/guide/getting-started' },
      { text: 'The editor', link: '/en/editor' },
    ],
  },
  {
    text: 'Scripting (Rust)',
    items: [
      { text: 'Overview and cheat sheet', link: '/en/scripting/' },
      { text: 'Writing scripts the easy way', link: '/en/scripting/easy' },
      { text: 'Vertices, UVs, smoothing', link: '/en/scripting/mesh' },
      { text: 'Cookbook', link: '/en/scripting/cookbook' },
      { text: 'Architecture: scripts per object, signals, build', link: '/en/scripting/architecture' },
      { text: 'Game systems: UI, saves, cameras, sound, decals', link: '/en/scripting/game' },
      { text: 'The LAST SERVICE game', link: '/en/guide/last-service' },
    ],
  },
  {
    text: 'Materials and nodes',
    items: [
      { text: 'Working with nodes', link: '/en/nodes/' },
      { text: 'Material catalogue', link: '/en/nodes/materials' },
      { text: 'How nodes connect', link: '/en/nodes/wiring' },
      { text: 'Node reference (table)', link: '/en/nodes/reference' },
      { text: 'Shader and script catalogue', link: '/en/library' },
    ],
  },
  {
    text: 'Nodes in detail',
    collapsed: true,
    items: [
      { text: 'Input', link: '/en/nodes/ref/input' },
      { text: 'Geometry', link: '/en/nodes/ref/geometry' },
      { text: 'Light', link: '/en/nodes/ref/light' },
      { text: 'Normals', link: '/en/nodes/ref/normal' },
      { text: 'Noise', link: '/en/nodes/ref/noise' },
      { text: 'Patterns', link: '/en/nodes/ref/procedural' },
      { text: 'Colour', link: '/en/nodes/ref/colour' },
      { text: 'Math', link: '/en/nodes/ref/math' },
      { text: 'Vectors', link: '/en/nodes/ref/vector' },
      { text: 'Utility', link: '/en/nodes/ref/utility' },
      { text: 'Camera', link: '/en/nodes/ref/effect' },
    ],
  },
]

export default defineConfig({
  title: 'Kvas',
  description: 'A Rust 3D game engine: editor, node materials, hot-reloaded scripts.',
  base,
  cleanUrls: true,
  lastUpdated: true,
  markdown: { languageAlias: { 'rust,no_run': 'rust', 'rust,ignore': 'rust', 'rust,should_panic': 'rust' } },
  srcExclude: ['ASSETS.md', 'README.md', 'node_modules/**'],
  head: [
    ['link', { rel: 'icon', href: `${base}logo.svg` }],
    ['meta', { name: 'theme-color', content: '#3574f0' }],
    ['meta', { property: 'og:image', content: 'https://nopass0.github.io/kvas/images/demo.png' }],
  ],
  locales: {
    root: {
      label: 'Русский',
      lang: 'ru-RU',
      description: 'Игровой 3D-движок на Rust: редактор, нодовые материалы, скрипты с горячей перезагрузкой.',
      themeConfig: {
        nav: [
          { text: 'Начало', link: '/guide/getting-started' },
          { text: 'Скрипты', link: '/scripting/' },
          { text: 'Ноды и материалы', link: '/nodes/' },
          { text: 'Руководства', link: '/GUIDE' },
          { text: 'Rust API', link: `${base}api/kvas_engine/index.html`, target: '_self' },
        ],
        sidebar: ruSidebar,
        outline: { label: 'На этой странице', level: [2, 3] },
        docFooter: { prev: 'Назад', next: 'Дальше' },
        lastUpdated: { text: 'Обновлено' },
        returnToTopLabel: 'Наверх',
        sidebarMenuLabel: 'Меню',
        darkModeSwitchLabel: 'Тема',
        langMenuLabel: 'Язык',
        editLink: { pattern: 'https://github.com/Nopass0/kvas/edit/main/docs/:path', text: 'Править эту страницу' },
      },
    },
    en: {
      label: 'English',
      lang: 'en-US',
      link: '/en/',
      description: 'A Rust 3D game engine: editor, node materials, hot-reloaded scripts.',
      themeConfig: {
        nav: [
          { text: 'Start', link: '/en/guide/getting-started' },
          { text: 'Scripting', link: '/en/scripting/' },
          { text: 'Nodes and materials', link: '/en/nodes/' },
          { text: 'Rust API', link: `${base}api/kvas_engine/index.html`, target: '_self' },
        ],
        sidebar: enSidebar,
        outline: { label: 'On this page', level: [2, 3] },
        editLink: { pattern: 'https://github.com/Nopass0/kvas/edit/main/docs/:path', text: 'Edit this page' },
      },
    },
  },
  themeConfig: {
    logo: '/logo.svg',
    siteTitle: 'Kvas',
    socialLinks: [{ icon: 'github', link: 'https://github.com/Nopass0/kvas' }],
    search,
    footer: { message: 'Kvas Engine', copyright: 'License: see LICENSE in the repository' },
  },
})
