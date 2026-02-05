import { create } from 'zustand'
import { persist } from 'zustand/middleware'
import * as profileApi from '@/services/profileApi'
import type { UserProfile, UpdateProfileRequest } from '@/types/auth'

interface ProfileState {
  profile: UserProfile | null
  isLoading: boolean
  error: string | null

  // Actions
  fetchProfile: () => Promise<void>
  updateProfile: (req: UpdateProfileRequest) => Promise<void>
  syncProfile: () => Promise<void>
  clearProfile: () => void
}

export const useProfileStore = create<ProfileState>()(
  persist(
    (set) => ({
      profile: null,
      isLoading: false,
      error: null,

      fetchProfile: async () => {
        set({ isLoading: true, error: null })
        try {
          const profile = await profileApi.getUserProfile()
          set({ profile, isLoading: false })
        } catch (error) {
          set({
            error: error instanceof Error ? error.message : '获取资料失败',
            isLoading: false,
          })
        }
      },

      updateProfile: async (req) => {
        set({ isLoading: true, error: null })
        try {
          const profile = await profileApi.updateUserProfile(req)
          set({ profile, isLoading: false })
        } catch (error) {
          set({
            error: error instanceof Error ? error.message : '更新失败',
            isLoading: false,
          })
          throw error
        }
      },

      syncProfile: async () => {
        set({ isLoading: true, error: null })
        try {
          const profile = await profileApi.syncProfile()
          set({ profile, isLoading: false })
        } catch (error) {
          set({
            error: error instanceof Error ? error.message : '同步失败',
            isLoading: false,
          })
          throw error
        }
      },

      clearProfile: () => set({ profile: null }),
    }),
    {
      name: 'profile-storage',
      partialize: (state) => ({
        profile: state.profile,
      }),
    }
  )
)
