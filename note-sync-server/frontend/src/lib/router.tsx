import { createBrowserRouter, Navigate } from 'react-router-dom'
import { ProtectedRoute } from '@/components/ProtectedRoute'
import AdminPanel from '@/components/sync/AdminPanel'
import { Login } from '@/pages/Login'
import { Register } from '@/pages/Register'

export const router = createBrowserRouter([
  {
    path: '/',
    element: (
      <ProtectedRoute>
        <AdminPanel />
      </ProtectedRoute>
    ),
  },
  {
    path: '/login',
    element: <Login />,
  },
  {
    path: '/register',
    element: <Register />,
  },
  {
    path: '*',
    element: <Navigate to="/" replace />,
  },
])
