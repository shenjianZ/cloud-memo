import axios, { AxiosError, type AxiosInstance } from 'axios'

const BASE_URL = import.meta.env.VITE_API_URL || 'http://localhost:3000'

// 创建 axios 实例
export const apiClient: AxiosInstance = axios.create({
  baseURL: BASE_URL,
  timeout: 10000,
  headers: {
    'Content-Type': 'application/json',
  },
})

// 请求拦截器：自动添加 JWT token
apiClient.interceptors.request.use(
  (config) => {
    const token = localStorage.getItem('auth_token')
    if (token) {
      config.headers.Authorization = `Bearer ${token}`
    }
    return config
  },
  (error) => {
    return Promise.reject(error)
  }
)

// 响应拦截器：统一错误处理
apiClient.interceptors.response.use(
  (response) => response,
  (error: AxiosError) => {
    if (error.response) {
      switch (error.response.status) {
        case 401:
          // Token 过期或无效，清除 token 并跳转登录
          localStorage.removeItem('auth_token')
          window.location.href = '/login'
          break
        case 403:
          console.error('没有权限执行此操作')
          break
        case 409:
          console.error('数据冲突，请刷新后重试')
          break
        case 500:
          console.error('服务器错误，请稍后重试')
          break
        default:
          console.error('请求失败:', error.message)
      }
    } else if (error.request) {
      console.error('网络错误，请检查连接')
    } else {
      console.error('请求配置错误:', error.message)
    }
    return Promise.reject(error)
  }
)

export default apiClient
