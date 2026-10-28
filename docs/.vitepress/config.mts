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
      { text: 'Rust API: руководство', link: '/ENGINE_API' },
    ],
  },
  {
    text: 'Материалы и ноды',
    items: [
      { text: 'Как работать с нодами', link: '/nodes/' },
      { text: 'Каталог материалов', link: '/nodes/materials' },
      { text: 'Справочник нод', link: '/nodes/reference' },
    ],
  },
  {
    text: 'Руководства',
    items: [
      { text: 'Большое руководство с примерами', link: '/GUIDE' },
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
    ],
  },
  {
    text: 'Materials and nodes',
    items: [
      { text: 'Working with nodes', link: '/en/nodes/' },
      { text: 'Material catalogue', link: '/en/nodes/materials' },
      { text: 'Node reference', link: '/en/nodes/reference' },
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
