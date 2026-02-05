import { create } from 'zustand'
import * as deviceApi from '@/api/devices'
import type { Device } from '@/api/devices'

interface DeviceState {
  devices: Device[]
  isLoading: boolean
  error: string | null

  fetchDevices: () => Promise<void>
  revokeDevice: (deviceId: string) => Promise<void>
  clearError: () => void
}

export const useDeviceStore = create<DeviceState>()((set) => ({
  devices: [],
  isLoading: false,
  error: null,

  fetchDevices: async () => {
    set({ isLoading: true, error: null })
    try {
      const devices = await deviceApi.listDevices()
      set({ devices, isLoading: false })
    } catch (error) {
      const message = error instanceof Error ? error.message : '获取设备列表失败'
      set({ error: message, isLoading: false })
      throw error
    }
  },

  revokeDevice: async (deviceId: string) => {
    set({ isLoading: true, error: null })
    try {
      await deviceApi.revokeDevice(deviceId)
      // 更新本地状态：将设备标记为已撤销
      set((state) => ({
        devices: state.devices.map((d) =>
          d.id === deviceId ? { ...d, revoked: true } : d
        ),
        isLoading: false,
      }))
    } catch (error) {
      const message = error instanceof Error ? error.message : '撤销设备失败'
      set({ error: message, isLoading: false })
      throw error
    }
  },

  clearError: () => set({ error: null }),
}))
