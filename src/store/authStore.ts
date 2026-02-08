import { create } from 'zustand'
import { persist } from 'zustand/middleware'
import * as authApi from '@/services/authApi'
import { migrateOrphanDataToWorkspace } from '@/services/workspaceApi'
import { useProfileStore } from './profileStore'
import { useWorkspaceStore } from './workspaceStore'
import { useNoteStore } from './noteStore'
import { useTagStore } from './tagStore'
import type { User, AccountWithProfile } from '@/types/auth'

interface AuthState {
  user: User | null
  isAuthenticated: boolean
  isLoading: boolean
  error: string | null
  allAccounts: AccountWithProfile[]  // 包含用户资料的账号列表

  // Actions
  login: (email: string, password: string, serverUrl?: string) => Promise<void>
  register: (email: string, password: string, serverUrl?: string) => Promise<void>
  logout: () => Promise<void>
  checkAuth: () => Promise<void>
  clearError: () => void

  // 新增：多账号管理
  listAccounts: () => Promise<void>
  switchAccount: (userId: string) => Promise<void>
  removeAccount: (userId: string) => Promise<void>

  // 新增：Token 自动刷新
  refreshAccessToken: () => Promise<void>
}

export const useAuthStore = create<AuthState>()(
  persist(
    (set, get) => ({
      user: null,
      isAuthenticated: false,
      isLoading: false,
      error: null,
      allAccounts: [],

      login: async (email: string, password: string, serverUrl?: string) => {
        set({ isLoading: true, error: null })
        try {
          // 如果 serverUrl 为空，传空字符串给后端，后端会使用默认服务器
          const serverUrlToUse = serverUrl || ''
          const response = await authApi.login({ email, password, serverUrl: serverUrlToUse })
          const user: User = {
            id: response.userId,
            email: response.email,
            serverUrl: serverUrlToUse,
            deviceId: '',
          }

          // ⚠️ 重要：登录成功后，像 switchAccount 一样加载所有业务数据
          console.log('[authStore] 登录成功，开始加载业务数据')

          // 1. 加载用户资料
          const profileStore = useProfileStore.getState()
          await profileStore.fetchProfile()
          console.log('[authStore] 用户资料加载成功')

          // 2. 加载工作空间（必须先加载，因为其他数据依赖它）
          const workspaceStore = useWorkspaceStore.getState()
          await workspaceStore.loadWorkspaces()
          await workspaceStore.loadCurrentWorkspace()
          console.log('[authStore] 工作空间加载成功，当前工作空间:', workspaceStore.currentWorkspace?.name)

          // 🆕 迁移孤立数据到当前工作空间（未登录时创建的数据）
          if (workspaceStore.currentWorkspace) {
            try {
              const migrateResult = await migrateOrphanDataToWorkspace(workspaceStore.currentWorkspace.id)
              console.log('[authStore] 孤立数据迁移完成:', migrateResult)
              if (migrateResult.notes > 0 || migrateResult.folders > 0 || migrateResult.tags > 0 || migrateResult.snapshots > 0) {
                console.log(`[authStore] 迁移统计: ${migrateResult.notes} 条笔记, ${migrateResult.folders} 个文件夹, ${migrateResult.tags} 个标签, ${migrateResult.snapshots} 个快照`)
              }
            } catch (migrateError) {
              console.warn('[authStore] 迁移孤立数据失败（非致命错误）:', migrateError)
              // 迁移失败不影响登录流程，继续执行
            }
          }

          // 3. 加载笔记和文件夹
          const noteStore = useNoteStore.getState()
          await noteStore.loadNotesFromStorage()
          console.log('[authStore] 笔记和文件夹加载成功')

          // 4. 加载标签
          const tagStore = useTagStore.getState()
          await tagStore.loadTags()
          console.log('[authStore] 标签加载成功')

          console.log('[authStore] 所有业务数据加载完成')

          // 5. 刷新账号列表
          await get().listAccounts()

          set({ user, isAuthenticated: true, isLoading: false })
          console.log('[authStore] 用户状态已更新')
        } catch (error) {
          // Tauri 返回的错误是 string 类型
          const errorMsg = typeof error === 'string' ? error : String(error)
          set({
            error: errorMsg,
            isLoading: false,
          })
          throw error
        }
      },

      register: async (email: string, password: string, serverUrl?: string) => {
        set({ isLoading: true, error: null })
        try {
          // 如果 serverUrl 为空，传空字符串给后端，后端会使用默认服务器
          const serverUrlToUse = serverUrl || ''

          console.log('[authStore] 准备调用 register API:', {
            email,
            password,
            serverUrl: serverUrlToUse,
            originalServerUrl: serverUrl
          })

          const response = await authApi.register({ email, password, serverUrl: serverUrlToUse })

          console.log('[authStore] register API 返回:', response)

          const user: User = {
            id: response.userId,
            email: response.email,
            serverUrl: serverUrlToUse,
            deviceId: '',
          }

          // ⚠️ 重要：注册成功后，像 switchAccount 一样加载所有业务数据
          console.log('[authStore] 注册成功，开始加载业务数据')

          // 1. 加载用户资料
          const profileStore = useProfileStore.getState()
          await profileStore.fetchProfile()
          console.log('[authStore] 用户资料加载成功')

          // 2. 加载工作空间（必须先加载，因为其他数据依赖它）
          const workspaceStore = useWorkspaceStore.getState()
          await workspaceStore.loadWorkspaces()
          await workspaceStore.loadCurrentWorkspace()
          console.log('[authStore] 工作空间加载成功，当前工作空间:', workspaceStore.currentWorkspace?.name)

          // 🆕 迁移孤立数据到当前工作空间（未登录时创建的数据）
          if (workspaceStore.currentWorkspace) {
            try {
              const migrateResult = await migrateOrphanDataToWorkspace(workspaceStore.currentWorkspace.id)
              console.log('[authStore] 孤立数据迁移完成:', migrateResult)
              if (migrateResult.notes > 0 || migrateResult.folders > 0 || migrateResult.tags > 0 || migrateResult.snapshots > 0) {
                console.log(`[authStore] 迁移统计: ${migrateResult.notes} 条笔记, ${migrateResult.folders} 个文件夹, ${migrateResult.tags} 个标签, ${migrateResult.snapshots} 个快照`)
              }
            } catch (migrateError) {
              console.warn('[authStore] 迁移孤立数据失败（非致命错误）:', migrateError)
              // 迁移失败不影响注册流程，继续执行
            }
          }

          // 3. 加载笔记和文件夹
          const noteStore = useNoteStore.getState()
          await noteStore.loadNotesFromStorage()
          console.log('[authStore] 笔记和文件夹加载成功')

          // 4. 加载标签
          const tagStore = useTagStore.getState()
          await tagStore.loadTags()
          console.log('[authStore] 标签加载成功')

          console.log('[authStore] 所有业务数据加载完成')

          // 5. 刷新账号列表
          await get().listAccounts()

          set({ user, isAuthenticated: true, isLoading: false })
          console.log('[authStore] 用户状态已更新')
        } catch (error) {
          // Tauri 返回的错误是 string 类型
          const errorMsg = typeof error === 'string' ? error : String(error)
          console.error('[authStore] register 错误:', errorMsg)
          set({
            error: errorMsg,
            isLoading: false,
          })
          throw error
        }
      },

      logout: async () => {
        set({ isLoading: true, error: null })
        try {
          await authApi.logout()

          // 清空当前用户，但保留账号列表
          set({
            user: null,
            isAuthenticated: false,
            isLoading: false,
            allAccounts: []  // 登出时清空账号列表
          })
        } catch (error) {
          set({
            error: error instanceof Error ? error.message : '登出失败',
            isLoading: false,
          })
        }
      },

      checkAuth: async () => {
        try {
          const isAuth = await authApi.isAuthenticated()
          if (isAuth) {
            const user = await authApi.getCurrentUser()
            set({ user, isAuthenticated: true })

            // 同时获取所有账号列表
            get().listAccounts()
          } else {
            // Token 可能已过期，尝试刷新
            console.log('[authStore] Token expired or not authenticated, attempting refresh...')
            try {
              await get().refreshAccessToken()
              const user = await authApi.getCurrentUser()
              set({ user, isAuthenticated: true })
              get().listAccounts()
            } catch (refreshError) {
              console.error('[authStore] Refresh failed:', refreshError)
              set({ user: null, isAuthenticated: false, allAccounts: [] })
            }
          }
        } catch (error) {
          console.error('[authStore] checkAuth error:', error)
          set({ user: null, isAuthenticated: false, allAccounts: [] })
        }
      },

      clearError: () => set({ error: null }),

      // 新增：获取所有账号列表
      listAccounts: async () => {
        try {
          const accounts = await authApi.listAccounts()
          set({ allAccounts: accounts })
        } catch (error) {
          console.error('[authStore] 获取账号列表失败:', error)
          set({ allAccounts: [] })
        }
      },

      // 新增：切换账号
      switchAccount: async (userId: string) => {
        console.log('[authStore] switchAccount 开始: userId =', userId)
        console.log('[authStore] 当前用户:', get().user)
        set({ isLoading: true, error: null })
        try {
          await authApi.switchAccount(userId)
          console.log('[authStore] 后端 switch_account 调用成功')

          // 重新获取当前用户和账号列表
          const user = await authApi.getCurrentUser()
          const accounts = await authApi.listAccounts()

          console.log('[authStore] 获取到的新用户:', user)
          console.log('[authStore] 获取到的账号列表:', accounts)

          // 清空旧的用户资料并重新获取新账号的资料
          console.log('[authStore] 清空旧用户资料并获取新用户资料')
          const profileStore = useProfileStore.getState()
          profileStore.clearProfile()
          await profileStore.fetchProfile()
          console.log('[authStore] 新用户资料获取成功:', profileStore.profile)

          // ⚠️ 重要：清空并重新加载所有业务数据
          console.log('[authStore] 开始重新加载业务数据')

          // 1. 清空并重新加载工作空间（必须先加载，因为其他数据依赖它）
          const workspaceStore = useWorkspaceStore.getState()
          workspaceStore.clearWorkspaceState()  // 先清空状态，避免显示旧账号的工作空间
          await workspaceStore.loadWorkspaces()
          await workspaceStore.loadCurrentWorkspace()  // 加载新账号的当前工作空间
          console.log('[authStore] 工作空间加载成功，当前工作空间:', workspaceStore.currentWorkspace?.name)

          // 2. 重新加载笔记和文件夹
          const noteStore = useNoteStore.getState()
          noteStore.clearNotesState()  // 只清空前端状态，不删除数据库
          await noteStore.loadNotesFromStorage()
          console.log('[authStore] 笔记和文件夹加载成功')

          // 3. 重新加载标签
          const tagStore = useTagStore.getState()
          await tagStore.loadTags()  // loadTags 会直接覆盖，无需先清空
          console.log('[authStore] 标签加载成功')

          console.log('[authStore] 所有业务数据加载完成')

          set({
            user,
            isAuthenticated: true,
            isLoading: false,
            allAccounts: accounts
          })
          console.log('[authStore] 状态已更新')
        } catch (error) {
          console.error('[authStore] switchAccount 错误:', error)
          set({
            error: error instanceof Error ? error.message : '切换账号失败',
            isLoading: false,
          })
          throw error
        }
      },

      // 新增：删除账号
      removeAccount: async (userId: string) => {
        set({ isLoading: true, error: null })
        try {
          await authApi.removeAccount(userId)

          // 刷新账号列表
          const accounts = await authApi.listAccounts()
          set({ allAccounts: accounts, isLoading: false })
        } catch (error) {
          set({
            error: error instanceof Error ? error.message : '删除账号失败',
            isLoading: false,
          })
          throw error
        }
      },

      // 新增：Token 自动刷新
      refreshAccessToken: async () => {
        set({ isLoading: true, error: null })
        try {
          const response = await authApi.refreshAccessToken()

          // 更新用户状态
          const user: User = {
            id: response.userId,
            email: response.email,
            serverUrl: '', // 从本地获取
            deviceId: '',
            lastSyncAt: undefined,
          }

          set({
            user,
            isAuthenticated: true,
            isLoading: false,
          })

          console.log('[authStore] Token refreshed successfully')
        } catch (error) {
          console.error('[authStore] Token refresh failed:', error)
          set({
            error: error instanceof Error ? error.message : 'Token 刷新失败',
            isLoading: false,
          })
          throw error
        }
      },
    }),
    {
      name: 'auth-storage',
      partialize: (state) => ({
        user: state.user,
        isAuthenticated: state.isAuthenticated,
        // 不持久化 allAccounts，每次启动时重新获取
      }),
    }
  )
)
