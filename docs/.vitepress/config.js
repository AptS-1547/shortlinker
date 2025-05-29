export default {
  title: "Shortlinker",
  description: "基于 Rust 构建的极简 URL 短链接服务，支持 HTTP 302 重定向，易于部署且性能卓越",
  lang: 'zh-CN',
  head: [
    ['link', { rel: 'icon', href: '/favicon.ico' }]
  ],
  
  themeConfig: {
    logo: '/logo.svg',
    
    nav: [
      { text: '首页', link: '/' },
      { text: '项目介绍', link: '/guide/' },
      { text: '快速开始', link: '/guide/getting-started' },
      { text: 'CLI 工具', link: '/cli/' },
      { text: '配置说明', link: '/config/' },
      { text: '部署指南', link: '/deployment/' },
      { text: 'API 文档', link: '/api/' }
    ],

    sidebar: {
      '/guide/': [
        {
          text: '开始使用',
          items: [
            { text: '项目介绍', link: '/guide/' },
            { text: '安装指南', link: '/guide/installation' },
            { text: '快速开始', link: '/guide/getting-started' }
          ]
        }
      ],
      '/cli/': [
        {
          text: 'CLI 工具',
          items: [
            { text: 'CLI 概述', link: '/cli/' },
            { text: '命令参考', link: '/cli/commands' }
          ]
        }
      ],
      '/config/': [
        {
          text: '配置说明',
          items: [
            { text: '环境变量', link: '/config/' },
            { text: '配置示例', link: '/config/examples' }
          ]
        }
      ],
      '/deployment/': [
        {
          text: '部署指南',
          items: [
            { text: '部署概述', link: '/deployment/' },
            { text: 'Docker 部署', link: '/deployment/docker' },
            { text: '反向代理', link: '/deployment/proxy' },
            { text: '系统服务', link: '/deployment/systemd' }
          ]
        }
      ],
      '/api/': [
        {
          text: 'API 文档',
          items: [
            { text: 'HTTP 接口', link: '/api/' },
            { text: 'Admin API', link: '/api/admin' }
          ]
        }
      ]
    },

    socialLinks: [
      { icon: 'github', link: 'https://github.com/AptS-1547/shortlinker' }
    ],

    footer: {
      message: '基于 MIT 许可证发布',
      copyright: 'Copyright © 2024-present AptS:1547'
    },

    search: {
      provider: 'local'
    }
  }
}
