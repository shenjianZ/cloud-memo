import { WebviewWindow, getAllWebviewWindows } from '@tauri-apps/api/webviewWindow'

/**
 * 打开/显示登录窗口
 * 如果窗口存在则显示，如果不存在（被关闭了）则创建新的
 */
export async function openAuthWindow() {
  try {
    // 1. 尝试获取已存在的窗口
    const windows = await getAllWebviewWindows()
    const existingWin = windows.find(w => w.label === 'auth')

    if (existingWin) {
      // 2. 窗口存在，直接显示
      await existingWin.show()
      await existingWin.unminimize()
      await existingWin.setFocus()
      console.log('Auth window shown successfully')
    } else {
      // 3. 窗口不存在（可能被 close 了），重新创建
      console.log('Auth window not found, creating new instance...')

      const newWin = new WebviewWindow('auth', {
        url: 'index.html#/auth',
        width: 400,
        height: 520,
        minWidth: 400,
        minHeight: 520,
        maxWidth: 400,
        maxHeight: 520,
        resizable: false,
        center: true,
        decorations: false,
        shadow: true,
        alwaysOnTop: true,
        skipTaskbar: true,
        title: '登录 - Markdown Notes',
        devtools: true,  // ✨ 启用开发者工具
      })

      // 监听创建成功
      newWin.once('tauri://created', () => {
        console.log('✅ Auth window created successfully')
      })

      // 监听创建失败
      newWin.once('tauri://error', (e) => {
        console.error('Failed to create auth window:', e)
      })
    }
  } catch (e) {
    console.error('Failed to open auth window:', e)
  }
}

/**
 * 关闭/隐藏登录窗口
 * 使用 hide 而不是 close，这样窗口保持活跃状态，下次打开更快
 */
export async function closeAuthWindow() {
  try {
    const windows = await getAllWebviewWindows()
    const authWin = windows.find(w => w.label === 'auth')

    if (authWin) {
      // 使用 hide 而不是 close，保持窗口实例
      await authWin.hide()
      console.log('Auth window hidden')
    }
  } catch (e) {
    console.error('Failed to close auth window:', e)
  }
}

