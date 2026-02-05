import apiClient from './client'

export interface Device {
  id: string
  user_id: string
  device_name: string
  device_type: 'desktop' | 'laptop' | 'mobile' | 'tablet'
  revoked: boolean
  last_seen_at: number
  created_at: number
}

export interface RegisterDeviceRequest {
  device_name: string
  device_type?: string
}

/**
 * 获取用户的所有设备
 */
export async function listDevices(): Promise<Device[]> {
  const response = await apiClient.get<Device[]>('/devices')
  return response.data
}

/**
 * 注册新设备（登录时自动调用）
 */
export async function registerDevice(req: RegisterDeviceRequest): Promise<Device> {
  const response = await apiClient.post<Device>('/devices/register', req)
  return response.data
}

/**
 * 撤销设备
 */
export async function revokeDevice(deviceId: string): Promise<void> {
  await apiClient.delete(`/devices/${deviceId}`)
}

/**
 * 发送设备心跳
 */
export async function sendHeartbeat(deviceId: string): Promise<void> {
  await apiClient.post(`/devices/${deviceId}/heartbeat`)
}

/**
 * 解析设备类型（根据 User-Agent）
 */
export function parseDeviceType(userAgent: string): 'desktop' | 'laptop' | 'mobile' | 'tablet' {
  const ua = userAgent.toLowerCase()

  if (ua.includes('iphone') || (ua.includes('macintosh') && 'ontouchend' in document)) {
    return 'mobile'
  }
  if (ua.includes('ipad')) {
    return 'tablet'
  }
  if (ua.includes('android')) {
    if (!ua.includes('mobile')) {
      return 'tablet'
    }
    return 'mobile'
  }
  if (ua.includes('windows') || ua.includes('macintosh') || ua.includes('linux')) {
    // 检测是否是笔记本（简化判断）
    if (ua.includes('laptop') || ua.includes('notebook')) {
      return 'laptop'
    }
    return 'desktop'
  }

  return 'desktop' // 默认
}
