import { useEffect } from 'react'
import { ThemeProvider } from './components/theme-provider'
import { Toaster } from 'sonner'
import { HashRouter, useNavigate } from 'react-router-dom'
import AppRoutes from './routes'
import { useSidebarStore } from './store/sidebarStore'
import { globalKeyHandler } from './lib/globalKeyHandler'
import { useInitEditorSettings } from './hooks/useEditorFontSettings'

// 订阅路由变化以更新快捷键处理器的当前路径
function RouteWatcher() {
  const navigate = useNavigate()

  useEffect(() => {
    // 监听路由变化
    const unlisten = () => {
      const hash = window.location.hash.slice(1) || '/'
      globalKeyHandler.setCurrentPath(hash)
    }

    // 初始化当前路径
    unlisten()

    // 监听 hashchange 事件
    window.addEventListener('hashchange', unlisten)

    return () => {
      window.removeEventListener('hashchange', unlisten)
    }
  }, [navigate])

  return null
}

function App() {
  const navigate = useNavigate()
  const toggleSidebar = useSidebarStore((state) => state.toggleSidebar)

  // 初始化编辑器设置
  useInitEditorSettings()

  useEffect(() => {
    // 全局快捷键监听器
    const handleToggleSidebar = () => {
      toggleSidebar()
    }

    const handleOpenSettings = () => {
      navigate('/settings')
    }

    window.addEventListener('keybinding-toggle-sidebar', handleToggleSidebar)
    window.addEventListener('keybinding-open-settings', handleOpenSettings)

    return () => {
      window.removeEventListener('keybinding-toggle-sidebar', handleToggleSidebar)
      window.removeEventListener('keybinding-open-settings', handleOpenSettings)
    }
  }, [navigate, toggleSidebar])

  useEffect(() => {
    // 禁用全局浏览器右键菜单
    const handleContextMenu = (e: MouseEvent) => {
      e.preventDefault()
    }

    document.addEventListener('contextmenu', handleContextMenu)

    return () => {
      document.removeEventListener('contextmenu', handleContextMenu)
    }
  }, [])

  return (
    <>
      <RouteWatcher />
      <AppRoutes />
    </>
  )
}

function AppWrapper() {
  return (
    <ThemeProvider defaultTheme="system" storageKey="markdown-notes-ui-theme">
      <HashRouter>
        <App />
      </HashRouter>
      <Toaster position="top-right" />
    </ThemeProvider>
  )
}

export default AppWrapper
