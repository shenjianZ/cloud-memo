import { useState } from 'react'
import { useNavigate } from 'react-router-dom'
import { useAuthStore } from '@/store/authStore'
import { useDevice } from '@/hooks/useDevice'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Link } from 'react-router-dom'
import { toast } from 'react-toastify'

export function Register() {
  const [email, setEmail] = useState('')
  const [password, setPassword] = useState('')
  const [confirmPassword, setConfirmPassword] = useState('')
  const [passwordError, setPasswordError] = useState('')
  const { register, isLoading, error, clearError } = useAuthStore()
  const { registerCurrentDevice, startHeartbeat } = useDevice()
  const navigate = useNavigate()

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    clearError()
    setPasswordError('')

    // 验证密码
    if (password.length < 6) {
      setPasswordError('密码至少需要6个字符')
      toast.error('密码至少需要6个字符')
      return
    }

    if (password !== confirmPassword) {
      setPasswordError('密码不匹配')
      toast.error('密码不匹配')
      return
    }

    try {
      await register(email, password)

      // 注册成功后注册当前设备
      try {
        await registerCurrentDevice()
        startHeartbeat()
      } catch (deviceError) {
        console.error('Device registration failed:', deviceError)
      }

      toast.success('注册成功')
      navigate('/', { replace: true })
    } catch (err) {
      // Error handled by store
      toast.error(error || '注册失败')
    }
  }

  return (
    <div className="container flex items-center justify-center min-h-screen">
      <Card className="w-full max-w-md">
        <CardHeader>
          <CardTitle>注册</CardTitle>
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
                placeholder="至少6个字符"
                value={password}
                onChange={(e) => setPassword(e.target.value)}
                required
                disabled={isLoading}
                minLength={6}
              />
            </div>
            <div>
              <label htmlFor="confirmPassword" className="block text-sm font-medium mb-2">
                确认密码
              </label>
              <Input
                id="confirmPassword"
                type="password"
                placeholder="再次输入密码"
                value={confirmPassword}
                onChange={(e) => setConfirmPassword(e.target.value)}
                required
                disabled={isLoading}
              />
            </div>
            {(error || passwordError) && (
              <div className="text-sm text-red-500 bg-red-50 dark:bg-red-950/20 p-3 rounded">
                {passwordError || error}
              </div>
            )}
            <Button
              type="submit"
              disabled={isLoading}
              className="w-full"
            >
              {isLoading ? '注册中...' : '注册'}
            </Button>
          </form>
          <div className="mt-4 text-center text-sm text-muted-foreground">
            已有账号？{' '}
            <Link
              to="/login"
              className="text-primary hover:underline font-medium"
            >
              登录
            </Link>
          </div>
        </CardContent>
      </Card>
    </div>
  )
}
