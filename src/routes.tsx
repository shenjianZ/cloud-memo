import { Routes, Route, Navigate } from 'react-router-dom'
import { MainLayout } from './components/layout/MainLayout'
import Home from './pages/Home'
import { Editor } from './pages/Editor'
import AllNotes from './pages/AllNotes'
import Favorites from './pages/Favorites'
import Settings from './pages/Settings'
import Trash from './pages/Trash'
import Profile from './pages/Profile'
import Auth from './pages/Auth'

export default function AppRoutes() {
  return (
    <Routes>
      {/* Auth 窗口的独立路由（必须在最前面） */}
      <Route path="/auth" element={<Auth />} />

      {/* 主窗口路由 */}
      <Route path="/" element={<MainLayout />}>
        <Route index element={<Home />} />
        <Route path="editor/:noteId" element={<Editor />} />
        <Route path="notes" element={<AllNotes />} />
        <Route path="favorites" element={<Favorites />} />
        <Route path="trash" element={<Trash />} />
        <Route path="settings" element={<Settings />} />
        <Route path="profile" element={<Profile />} />
        <Route path="*" element={<Navigate to="/" replace />} />
      </Route>

      {/* 兜底：如果连 /auth 都没匹配到，去首页 */}
      <Route path="*" element={<Navigate to="/" replace />} />
    </Routes>
  )
}
