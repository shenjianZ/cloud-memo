import { Routes, Route, Navigate } from 'react-router-dom'
import { MainLayout } from './components/layout/MainLayout'
import Home from './pages/Home'
import { Editor } from './pages/Editor'
import AllNotes from './pages/AllNotes'
import Favorites from './pages/Favorites'
import Settings from './pages/Settings'

export default function AppRoutes() {
  return (
    <Routes>
      <Route path="/" element={<MainLayout />}>
        <Route index element={<Home />} />
        <Route path="editor/:noteId" element={<Editor />} />
        <Route path="notes" element={<AllNotes />} />
        <Route path="favorites" element={<Favorites />} />
        <Route path="settings" element={<Settings />} />
        <Route path="*" element={<Navigate to="/" replace />} />
      </Route>
    </Routes>
  )
}
