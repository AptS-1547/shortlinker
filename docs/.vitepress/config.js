import { readFileSync } from 'fs'
import { resolve } from 'path'

// 动态读取 Cargo.toml 版本号
function getVersion() {
  try {
    const cargoPath = resolve(__dirname, '../../Cargo.toml')
    const content = readFileSync(cargoPath, 'utf-8')
    const match = content.match(/^version\s*=\s*"([^"]+)"/m)
    return match ? match[1] : 'unknown'
  } catch {
    return 'unknown'
  }
}

const VERSION = getVersion()

export default {
  title: "Shortlinker",
  description: "基于 Rust 构建的极简 URL 短链接服务，支持 HTTP 302 重定向，易于部署且性能卓越",
  
  locales: {
    root: {
      label: '简体中文',
      lang: 'zh-CN',
      title: "Shortlinker",
      description: "基于 Rust 构建的极简 URL 短链接服务，支持 HTTP 302 重定向，易于部署且性能卓越",
      themeConfig: {
        nav: [
          { text: '首页', link: '/' },
          { text: '快速开始', link: '/guide/getting-started' },
          { text: '配置说明', link: '/config/' },
          { text: 'CLI 工具', link: '/cli/' },
          { text: 'Web 管理界面', link: '/admin-panel/' },
          { text: '部署指南', link: '/deployment/' },
          { text: 'API 文档', link: '/api/' },
          {
            text: `v${VERSION}`,
            items: [
              { text: '更新日志', link: 'https://github.com/AptS-1547/shortlinker/releases' },
              { text: 'GitHub', link: 'https://github.com/AptS-1547/shortlinker' }
            ]
          }
        ],
        footer: {
          message: '基于 MIT 许可证发布',
          copyright: 'Copyright © 2025 AptS:1547'
        },
        docFooter: {
          prev: '上一页',
          next: '下一页'
        },
        outline: {
          label: '页面导航'
        },
        lastUpdated: {
          text: '最后更新于',
          formatOptions: {
            dateStyle: 'short',
            timeStyle: 'medium'
          }
        },
        returnToTopLabel: '回到顶部',
        sidebarMenuLabel: '菜单',
        darkModeSwitchLabel: '主题',
        lightModeSwitchTitle: '切换到浅色模式',
        darkModeSwitchTitle: '切换到深色模式'
      }
    },
    en: {
      label: 'English',
      lang: 'en-US',
      title: "Shortlinker",
      description: "A minimalist URL shortening service built with Rust, supporting HTTP 302 redirects with easy deployment and excellent performance",
      themeConfig: {
        nav: [
          { text: 'Home', link: '/en/' },
          { text: 'Quick Start', link: '/en/guide/getting-started' },
          { text: 'Configuration', link: '/en/config/' },
          { text: 'CLI Tools', link: '/en/cli/' },
          { text: 'Web Admin Panel', link: '/en/admin-panel/' },
          { text: 'Deployment', link: '/en/deployment/' },
          { text: 'API Docs', link: '/en/api/' },
          {
            text: `v${VERSION}`,
            items: [
              { text: 'Changelog', link: 'https://github.com/AptS-1547/shortlinker/releases' },
              { text: 'GitHub', link: 'https://github.com/AptS-1547/shortlinker' }
            ]
          }
        ],
        footer: {
          message: 'Released under the MIT License',
          copyright: 'Copyright © 2025 AptS:1547'
        },
        docFooter: {
          prev: 'Previous page',
          next: 'Next page'
        },
        outline: {
          label: 'On this page'
        },
        lastUpdated: {
          text: 'Last updated',
          formatOptions: {
            dateStyle: 'short',
            timeStyle: 'medium'
          }
        }
      }
    }
  },
  
  head: [
    ['link', { rel: 'icon', href: '/favicon.ico' }],
    ['link', { rel: 'icon', type: 'image/svg+xml', href: '/logo.svg' }],
    ['link', { rel: 'apple-touch-icon', href: '/apple-touch-icon.png' }],
    ['meta', { name: 'theme-color', content: '#646cff' }],
    ['meta', { name: 'og:type', content: 'website' }],
    ['meta', { name: 'og:locale', content: 'zh_CN' }],
    ['meta', { name: 'og:title', content: 'Shortlinker | 极简 URL 短链接服务' }],
    ['meta', { name: 'og:site_name', content: 'Shortlinker' }],
    ['meta', { name: 'og:url', content: 'https://shortlinker.docs.ecaps.top' }],
  ],
  
  themeConfig: {
    logo: '/logo.svg',
    
    sidebar: {
      '/guide/': [
        {
          text: '开始使用',
          items: [
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
            { text: '命令参考', link: '/cli/commands' },
            { text: 'TUI 终端界面', link: '/cli/tui' }
          ]
        }
      ],
      '/config/': [
        {
          text: '配置说明',
          items: [
            { text: '环境变量', link: '/config/' },
            { text: '存储后端', link: '/config/storage' }
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
            { text: 'Admin API', link: '/api/admin' },
            { text: '健康检查 API', link: '/api/health' }
          ]
        }
      ],
      '/admin-panel/': [
        {
          text: 'Web 管理界面',
          items: [
            { text: '概述', link: '/admin-panel/' },
            { text: '开发指南', link: '/admin-panel/development' },
            { text: '故障排除', link: '/admin-panel/troubleshooting' }
          ]
        }
      ],
      '/cf-worker/': [
        {
          text: 'Cloudflare Worker',
          items: [
            { text: '概述', link: '/cf-worker/' }
          ]
        }
      ],

      // English sidebar
      '/en/': [
        {
          text: 'Getting Started',
          items: [
            { text: 'Installation', link: '/en/guide/installation' },
            { text: 'Quick Start', link: '/en/guide/getting-started' }
          ]
        }
      ],
      '/en/guide/': [
        {
          text: 'Getting Started',
          items: [
            { text: 'Installation', link: '/en/guide/installation' },
            { text: 'Quick Start', link: '/en/guide/getting-started' }
          ]
        }
      ],
      '/en/cli/': [
        {
          text: 'CLI Tools',
          items: [
            { text: 'CLI Overview', link: '/en/cli/' },
            { text: 'Command Reference', link: '/en/cli/commands' },
            { text: 'TUI Interface', link: '/en/cli/tui' }
          ]
        }
      ],
      '/en/config/': [
        {
          text: 'Configuration',
          items: [
            { text: 'Environment Variables', link: '/en/config/' },
            { text: 'Storage Backends', link: '/en/config/storage' }
          ]
        }
      ],
      '/en/deployment/': [
        {
          text: 'Deployment',
          items: [
            { text: 'Deployment Overview', link: '/en/deployment/' },
            { text: 'Docker Deployment', link: '/en/deployment/docker' },
            { text: 'Reverse Proxy', link: '/en/deployment/proxy' },
            { text: 'System Service', link: '/en/deployment/systemd' }
          ]
        }
      ],
      '/en/api/': [
        {
          text: 'API Documentation',
          items: [
            { text: 'HTTP Interface', link: '/en/api/' },
            { text: 'Admin API', link: '/en/api/admin' },
            { text: 'Health Check API', link: '/en/api/health' }
          ]
        }
      ],
      '/en/admin-panel/': [
        {
          text: 'Web Admin Panel',
          items: [
            { text: 'Overview', link: '/en/admin-panel/' },
            { text: 'Development Guide', link: '/en/admin-panel/development' },
            { text: 'Troubleshooting', link: '/en/admin-panel/troubleshooting' }
          ]
        }
      ],
      '/en/cf-worker/': [
        {
          text: 'Cloudflare Worker',
          items: [
            { text: 'Overview', link: '/en/cf-worker/' }
          ]
        }
      ]
    },

    socialLinks: [
      { icon: 'github', link: 'https://github.com/AptS-1547/shortlinker' }
    ],

    search: {
      provider: 'local',
      options: {
        locales: {
          zh: {
            translations: {
              button: {
                buttonText: '搜索文档',
                buttonAriaLabel: '搜索文档'
              },
              modal: {
                noResultsText: '无法找到相关结果',
                resetButtonTitle: '清除查询条件',
                footer: {
                  selectText: '选择',
                  navigateText: '切换'
                }
              }
            }
          }
        }
      }
    },

    editLink: {
      pattern: 'https://github.com/AptS-1547/shortlinker/edit/master/docs/:path',
      text: '编辑此页面'
    },

    lastUpdated: {
      text: '最后更新于',
      formatOptions: {
        dateStyle: 'short',
        timeStyle: 'medium'
      }
    }
  },

  markdown: {
    theme: {
      light: 'vitesse-light',
      dark: 'vitesse-dark'
    }
  }
}