import { create } from 'zustand'
import { persist } from 'zustand/middleware'
import * as authApi from '@/api/auth'

interface User {
  id: string
  email: string
}

interface AuthState {
  user: User | null
  isAuthenticated: boolean
  isLoading: boolean
  error: string | null

  login: (email: string, password: string) => Promise<void>
  register: (email: string, password: string) => Promise<void>
  logout: () => Promise<void>
  checkAuth: () => Promise<void>
  clearError: () => void
}

export const useAuthStore = create<AuthState>()(
  persist(
    (set) => ({
      user: null,
      isAuthenticated: false,
      isLoading: false,
      error: null,

      login: async (email: string, password: string) => {
        set({ isLoading: true, error: null })
        try {
          const response = await authApi.login({ email, password })
          const user: User = {
            id: response.user_id,
            email: response.email,
          }
          set({ user, isAuthenticated: true, isLoading: false })
        } catch (error) {
          const message = error instanceof Error ? error.message : '登录失败'
          set({ error: message, isLoading: false })
          throw error
        }
      },

      register: async (email: string, password: string) => {
        set({ isLoading: true, error: null })
        try {
          const response = await authApi.register({ email, password })
          const user: User = {
            id: response.user_id,
            email: response.email,
          }
          set({ user, isAuthenticated: true, isLoading: false })
        } catch (error) {
          const message = error instanceof Error ? error.message : '注册失败'
          set({ error: message, isLoading: false })
          throw error
        }
      },

      logout: async () => {
        set({ isLoading: true, error: null })
        try {
          await authApi.logout()
          set({
            user: null,
            isAuthenticated: false,
            isLoading: false,
          })
        } catch (error) {
          const message = error instanceof Error ? error.message : '登出失败'
          set({ error: message, isLoading: false })
          // 即使失败也清除本地状态
          set({
            user: null,
            isAuthenticated: false,
          })
        }
      },

      checkAuth: async () => {
        try {
          const user = await authApi.getCurrentUser()
          set({ user, isAuthenticated: true })
        } catch (error) {
          set({ user: null, isAuthenticated: false })
        }
      },

      clearError: () => set({ error: null }),
    }),
    {
      name: 'auth-storage',
      partialize: (state) => ({
        user: state.user,
        isAuthenticated: state.isAuthenticated,
      }),
    }
  )
)
