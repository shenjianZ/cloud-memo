import { useEffect } from 'react'
import { ThemeProvider } from './components/theme-provider'
import { Toaster } from 'sonner'
import { HashRouter, useNavigate } from 'react-router-dom'
import { listen } from '@tauri-apps/api/event'
import { getCurrentWindow } from '@tauri-apps/api/window'
import AppRoutes from './routes'
import { useSidebarStore } from './store/sidebarStore'
import { useAuthStore } from './store/authStore'
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
  const checkAuth = useAuthStore((state) => state.checkAuth)

  // 检查是否为主窗口（避免 auth 窗口重复初始化）
  useEffect(() => {
    const checkWindow = async () => {
      const currentWindow = await getCurrentWindow()
      const label = currentWindow.label
      const isMain = label === 'main'
      console.log('[App.tsx] 当前窗口 label:', label, '是否主窗口:', isMain)

      // 只在主窗口中初始化
      if (isMain) {
        console.log('[App.tsx] 主窗口初始化，开始检查认证状态')
        checkAuth()
      } else {
        console.log('[App.tsx] 非主窗口，跳过认证检查')
      }
    }
    checkWindow()
  }, [checkAuth])

  // 初始化编辑器设置（所有窗口都需要，但 store 有防重复逻辑）
  useInitEditorSettings()

  // 监听认证状态变化事件（从 Auth 窗口发送）
  useEffect(() => {
    const unlisten = listen('auth-state-changed', () => {
      console.log('[App.tsx] 收到 auth-state-changed 事件，刷新认证状态')
      checkAuth()
    })

    return () => {
      unlisten.then(fn => fn())
    }
  }, [checkAuth])

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
