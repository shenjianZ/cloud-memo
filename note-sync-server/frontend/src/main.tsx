import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import './index.css'
import App from './App.tsx'

createRoot(document.getElementById('root')!).render(
  <StrictMode>
    <App />
  </StrictMode>,
)

// 初始化：检查用户是否已登录
import { useAuthStore } from '@/store/authStore'
// 在应用启动时检查认证状态
if (localStorage.getItem('auth_token')) {
  useAuthStore.getState().checkAuth()
}

