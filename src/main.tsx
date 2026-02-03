import { createRoot } from 'react-dom/client'
import './index.css'
import App from './App'
import { globalKeyHandler } from './lib/globalKeyHandler'
// 导入 highlight.js 的代码高亮样式（亮色主题）
import 'highlight.js/styles/github.css'

// 初始化全局快捷键处理器
globalKeyHandler.initialize()

createRoot(document.getElementById('root')!).render(
  <App />
)
