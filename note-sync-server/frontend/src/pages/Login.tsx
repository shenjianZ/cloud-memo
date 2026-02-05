import { useState } from 'react'
import { useNavigate, useLocation } from 'react-router-dom'
import { useAuthStore } from '@/store/authStore'
import { useDevice } from '@/hooks/useDevice'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Link } from 'react-router-dom'
import { toast } from 'react-toastify'

export function Login() {
  const [email, setEmail] = useState('')
  const [password, setPassword] = useState('')
  const { login, isLoading, error, clearError } = useAuthStore()
  const { registerCurrentDevice, startHeartbeat } = useDevice()
  const navigate = useNavigate()
  const location = useLocation()

  const from = (location.state as any)?.from?.pathname || '/'

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    clearError()

    try {
      await login(email, password)

      // 登录成功后注册当前设备
      try {
        await registerCurrentDevice()
        startHeartbeat()
      } catch (deviceError) {
        console.error('Device registration failed:', deviceError)
        // 设备注册失败不影响登录流程
      }

      toast.success('登录成功')
      navigate(from, { replace: true })
    } catch (err) {
      // Error handled by store
      toast.error(error || '登录失败')
    }
  }

  return (
    <div className="container flex items-center justify-center min-h-screen">
      <Card className="w-full max-w-md">
        <CardHeader>
          <CardTitle>登录</CardTitle>
        </CardHeader>
        <CardContent>
          <form onSubmit={handleSubmit} className="space-y-4">
            <div>
              <label htmlFor="email" className="block text-sm font-medium mb-2">
                邮箱
              </label>
              <Input
                id="email"
                type="email"
                placeholder="user@example.com"
                value={email}
                onChange={(e) => setEmail(e.target.value)}
                required
                disabled={isLoading}
              />
            </div>
            <div>
              <label htmlFor="password" className="block text-sm font-medium mb-2">
                密码
              </label>
              <Input
                id="password"
                type="password"
                placeholder="••••••••"
                value={password}
                onChange={(e) => setPassword(e.target.value)}
                required
                disabled={isLoading}
              />
            </div>
            {error && (
              <div className="text-sm text-red-500 bg-red-50 dark:bg-red-950/20 p-3 rounded">
                {error}
              </div>
            )}
            <Button
              type="submit"
              disabled={isLoading}
              className="w-full"
            >
              {isLoading ? '登录中...' : '登录'}
            </Button>
          </form>
          <div className="mt-4 text-center text-sm text-muted-foreground">
            没有账号？{' '}
            <Link
              to="/register"
              className="text-primary hover:underline font-medium"
            >
              注册
            </Link>
          </div>
        </CardContent>
      </Card>
    </div>
  )
}
