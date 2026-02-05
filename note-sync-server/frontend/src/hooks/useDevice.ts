import { useEffect, useRef } from 'react'
import * as deviceApi from '@/api/devices'

/**
 * 设备管理 Hook
 * 自动处理设备注册和心跳
 */
export function useDevice() {
  const heartbeatTimer = useRef<NodeJS.Timeout | null>(null)

  /**
   * 注册当前设备（登录时调用）
   */
  const registerCurrentDevice = async () => {
    try {
      // 生成设备名称
      const deviceName = getDeviceName()

      // 解析设备类型
      const deviceType = deviceApi.parseDeviceType(navigator.userAgent)

      // 注册设备
      const device = await deviceApi.registerDevice({
        device_name: deviceName,
        device_type: deviceType,
      })

      // 保存设备 ID
      localStorage.setItem('current_device_id', device.id)

      console.log('Device registered:', device)
      return device
    } catch (error) {
      console.error('Failed to register device:', error)
      throw error
    }
  }

  /**
   * 开始心跳定时器（每 5 分钟）
   */
  const startHeartbeat = () => {
    const deviceId = localStorage.getItem('current_device_id')
    if (!deviceId) return

    // 清除现有定时器
    if (heartbeatTimer.current) {
      clearInterval(heartbeatTimer.current)
    }

    // 立即发送一次心跳
    sendHeartbeat(deviceId)

    // 每 5 分钟发送心跳
    heartbeatTimer.current = setInterval(() => {
      sendHeartbeat(deviceId)
    }, 5 * 60 * 1000) // 5 分钟
  }

  /**
   * 停止心跳定时器
   */
  const stopHeartbeat = () => {
    if (heartbeatTimer.current) {
      clearInterval(heartbeatTimer.current)
      heartbeatTimer.current = null
    }
  }

  /**
   * 发送心跳
   */
  const sendHeartbeat = async (deviceId: string) => {
    try {
      await deviceApi.sendHeartbeat(deviceId)
      console.log('Heartbeat sent successfully')
    } catch (error) {
      console.error('Failed to send heartbeat:', error)
    }
  }

  // 组件卸载时停止心跳
  useEffect(() => {
    return () => {
      stopHeartbeat()
    }
  }, [])

  return {
    registerCurrentDevice,
    startHeartbeat,
    stopHeartbeat,
  }
}

/**
 * 获取设备名称
 */
function getDeviceName(): string {
  const platform = navigator.platform
  const userAgent = navigator.userAgent

  // 检测浏览器
  let browser = 'Unknown Browser'
  if (userAgent.includes('Chrome') && !userAgent.includes('Edg')) {
    browser = 'Chrome'
  } else if (userAgent.includes('Firefox')) {
    browser = 'Firefox'
  } else if (userAgent.includes('Safari') && !userAgent.includes('Chrome')) {
    browser = 'Safari'
  } else if (userAgent.includes('Edg')) {
    browser = 'Edge'
  }

  // 检测操作系统
  let os = 'Unknown OS'
  if (platform.includes('Win')) {
    os = 'Windows'
  } else if (platform.includes('Mac')) {
    os = 'macOS'
  } else if (platform.includes('Linux')) {
    os = 'Linux'
  } else if (userAgent.includes('Android')) {
    os = 'Android'
  } else if (userAgent.includes('iPhone') || userAgent.includes('iPad')) {
    os = 'iOS'
  }

  return `${os} - ${browser}`
}
