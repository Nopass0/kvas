import { defineConfig } from 'vitepress'

// GitHub Pages serves the project at https://nopass0.github.io/kvas/
const base = process.env.DOCS_BASE ?? '/kvas/'

export default defineConfig({
  title: 'Kvas',
  description: 'Игровой 3D-движок на Rust: редактор, нодовые материалы, скрипты с горячей перезагрузкой.',
  lang: 'ru-RU',
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
  themeConfig: {
    logo: '/logo.svg',
    siteTitle: 'Kvas',
    nav: [
      { text: 'Начало', link: '/guide/getting-started' },
      { text: 'Скрипты', link: '/scripting/' },
      { text: 'Ноды', link: '/nodes/' },
      { text: 'Руководства', link: '/GUIDE' },
      { text: 'Rust API', link: `${base}api/kvas_engine/index.html`, target: '_self' },
    ],
    sidebar: [
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
          { text: 'Рецепты (cookbook)', link: '/scripting/cookbook' },
          { text: 'Rust API: руководство', link: '/ENGINE_API' },
        ],
      },
      {
        text: 'Нодовый редактор',
        items: [
          { text: 'Как работать с нодами', link: '/nodes/' },
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
    ],
    socialLinks: [{ icon: 'github', link: 'https://github.com/Nopass0/kvas' }],
    search: {
      provider: 'local',
      options: {
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
    editLink: { pattern: 'https://github.com/Nopass0/kvas/edit/main/docs/:path', text: 'Править эту страницу' },
    outline: { label: 'На этой странице', level: [2, 3] },
    docFooter: { prev: 'Назад', next: 'Дальше' },
    lastUpdated: { text: 'Обновлено' },
    returnToTopLabel: 'Наверх',
    sidebarMenuLabel: 'Меню',
    darkModeSwitchLabel: 'Тема',
    footer: { message: 'Kvas Engine', copyright: 'Лицензия — см. LICENSE в репозитории' },
  },
})
