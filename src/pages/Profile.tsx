import { useState, useEffect, useRef } from 'react'
import { useNavigate } from 'react-router-dom'
import { useAuthStore } from '@/store/authStore'
import { useProfileStore } from '@/store/profileStore'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Textarea } from '@/components/ui/textarea'
import { Label } from '@/components/ui/label'
import { toast } from 'sonner'
import { ArrowLeft, Save, Phone, MessageCircle, Upload } from 'lucide-react'
import { Avatar, AvatarFallback, AvatarImage } from '@/components/ui/avatar'

// 最大头像大小：5MB
const MAX_AVATAR_SIZE = 5 * 1024 * 1024
// 支持的图片格式
const SUPPORTED_FORMATS = ['image/jpeg', 'image/png', 'image/gif', 'image/webp', 'image/bmp', 'image/svg+xml']

/**
 * 将文件转换为 Base64
 */
function fileToBase64(file: File): Promise<string> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader()
    reader.onload = () => {
      const result = reader.result as string
      // 移除 data:image/xxx;base64, 前缀
      const base64 = result.split(',')[1]
      resolve(base64)
    }
    reader.onerror = reject
    reader.readAsDataURL(file)
  })
}

/**
 * 验证图片文件
 */
function validateImageFile(file: File): { valid: boolean; error?: string } {
  // 检查文件类型
  if (!SUPPORTED_FORMATS.includes(file.type)) {
    return {
      valid: false,
      error: `不支持的图片格式：${file.type}。仅支持 JPEG、PNG、GIF 格式`,
    }
  }

  // 检查文件大小
  if (file.size > MAX_AVATAR_SIZE) {
    return {
      valid: false,
      error: `图片过大：${(file.size / 1024 / 1024).toFixed(2)}MB（最大 5MB）`,
    }
  }

  return { valid: true }
}

export default function Profile() {
  const navigate = useNavigate()
  const { user } = useAuthStore()
  const { profile, isLoading, fetchProfile, updateProfile } = useProfileStore()
  const fileInputRef = useRef<HTMLInputElement>(null)

  const [formData, setFormData] = useState({
    username: '',
    phone: '',
    qq: '',
    wechat: '',
    bio: '',
    avatarData: '' as string | undefined,
    avatarMimeType: '' as string | undefined,
  })

  useEffect(() => {
    fetchProfile()
  }, [])

  useEffect(() => {
    if (profile) {
      setFormData({
        username: profile.username || '',
        phone: profile.phone || '',
        qq: profile.qq || '',
        wechat: profile.wechat || '',
        bio: profile.bio || '',
        avatarData: profile.avatarData,
        avatarMimeType: profile.avatarMimeType,
      })
    }
  }, [profile])

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    try {
      // 过滤掉空字符串的头像字段
      const submitData = {
        username: formData.username || undefined,
        phone: formData.phone || undefined,
        qq: formData.qq || undefined,
        wechat: formData.wechat || undefined,
        bio: formData.bio || undefined,
        avatarData: formData.avatarData || undefined,
        avatarMimeType: formData.avatarMimeType || undefined,
      }

      console.log('[Profile.tsx] 准备提交的数据:', {
        ...submitData,
        avatarData: submitData.avatarData ? `${submitData.avatarData.substring(0, 50)}... (${submitData.avatarData.length} chars)` : 'None',
        avatarMimeType: submitData.avatarMimeType || 'None'
      })

      await updateProfile(submitData)
      toast.success('保存成功', {
        description: '个人资料已更新',
      })
    } catch (error) {
      toast.error('保存失败', {
        description: error instanceof Error ? error.message : '未知错误',
      })
    }
  }

  const handleAvatarClick = () => {
    fileInputRef.current?.click()
  }

  const handleFileChange = async (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0]
    if (!file) return

    console.log('[Profile.tsx] 选择的文件:', {
      name: file.name,
      type: file.type,
      size: file.size
    })

    // 验证文件
    const validation = validateImageFile(file)
    if (!validation.valid) {
      toast.error('图片验证失败', {
        description: validation.error,
      })
      return
    }

    try {
      // 转换为 Base64
      const base64 = await fileToBase64(file)
      console.log('[Profile.tsx] Base64 转换成功:', {
        length: base64.length,
        preview: `${base64.substring(0, 50)}...`
      })

      setFormData(prev => ({
        ...prev,
        avatarData: base64,
        avatarMimeType: file.type,
      }))

      console.log('[Profile.tsx] formData 已更新')
      toast.success('图片已选择', {
        description: '点击"保存"按钮以更新头像',
      })
    } catch (error) {
      console.error('[Profile.tsx] 图片处理失败:', error)
      toast.error('图片处理失败', {
        description: error instanceof Error ? error.message : '未知错误',
      })
    }

    // 清空 input，允许重复选择同一文件
    e.target.value = ''
  }

  const getAvatarSrc = () => {
    if (formData.avatarData && formData.avatarMimeType) {
      return `data:${formData.avatarMimeType};base64,${formData.avatarData}`
    }
    return undefined
  }

  const handleChange = (field: string, value: string) => {
    setFormData(prev => ({ ...prev, [field]: value }))
  }

  const getInitials = (name: string) => {
    return name
      .split(' ')
      .map(n => n[0])
      .join('')
      .toUpperCase()
      .slice(0, 2)
  }

  const displayName = formData.username || user?.email?.split('@')[0] || 'User'
  const initials = getInitials(displayName)

  if (isLoading && !profile) {
    return (
      <div className="flex items-center justify-center h-full">
        <div className="text-muted-foreground">加载中...</div>
      </div>
    )
  }

  return (
    <div className="max-w-4xl mx-auto p-6 space-y-6">
      {/* 页面标题 */}
      <div className="flex items-center gap-4">
        <Button variant="ghost" size="icon" onClick={() => navigate(-1)}>
          <ArrowLeft className="h-4 w-4" />
        </Button>
        <div>
          <h1 className="text-3xl font-bold">个人中心</h1>
          <p className="text-sm text-muted-foreground">管理你的个人信息</p>
        </div>
      </div>

      {/* 头像区域 */}
      <div className="flex items-center gap-6 p-6 rounded-lg border bg-muted/20">
        <Avatar className="h-20 w-20">
          <AvatarImage src={getAvatarSrc()} />
          <AvatarFallback className="text-2xl font-medium">
            {initials}
          </AvatarFallback>
        </Avatar>
        <div className="flex-1">
          <h2 className="text-2xl font-semibold">{displayName}</h2>
          <p className="text-sm text-muted-foreground">{user?.email}</p>
        </div>
        <input
          ref={fileInputRef}
          type="file"
          accept="image/jpeg,image/png,image/gif,image/webp,image/bmp,image/svg+xml"
          onChange={handleFileChange}
          className="hidden"
        />
        <Button variant="outline" type="button" onClick={handleAvatarClick}>
          <Upload className="mr-2 h-4 w-4" />
          更换头像
        </Button>
      </div>

      {/* 编辑表单 */}
      <form onSubmit={handleSubmit} className="space-y-6">
        <div className="rounded-lg border p-6 space-y-4">
          <h3 className="text-xl font-semibold">基本信息</h3>

          <div className="space-y-2">
            <Label htmlFor="username">用户名</Label>
            <Input
              id="username"
              value={formData.username}
              onChange={(e) => handleChange('username', e.target.value)}
              placeholder="输入用户名"
            />
          </div>

          <div className="space-y-2">
            <Label htmlFor="email">邮箱</Label>
            <Input
              id="email"
              value={user?.email || ''}
              disabled
              className="bg-muted"
            />
            <p className="text-xs text-muted-foreground">
              邮箱不可修改，如需更改请联系客服
            </p>
          </div>

          <div className="space-y-2">
            <Label htmlFor="bio">个人简介</Label>
            <Textarea
              id="bio"
              value={formData.bio}
              onChange={(e) => handleChange('bio', e.target.value)}
              placeholder="介绍一下自己..."
              rows={3}
            />
          </div>
        </div>

        <div className="rounded-lg border p-6 space-y-4">
          <h3 className="text-xl font-semibold">联系方式</h3>

          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            <div className="space-y-2">
              <Label htmlFor="phone">
                <Phone className="inline mr-2 h-4 w-4" />
                手机号
              </Label>
              <Input
                id="phone"
                value={formData.phone}
                onChange={(e) => handleChange('phone', e.target.value)}
                placeholder="输入手机号"
              />
            </div>

            <div className="space-y-2">
              <Label htmlFor="qq">QQ</Label>
              <Input
                id="qq"
                value={formData.qq}
                onChange={(e) => handleChange('qq', e.target.value)}
                placeholder="输入 QQ 号"
              />
            </div>

            <div className="space-y-2">
              <Label htmlFor="wechat">
                <MessageCircle className="inline mr-2 h-4 w-4" />
                微信号
              </Label>
              <Input
                id="wechat"
                value={formData.wechat}
                onChange={(e) => handleChange('wechat', e.target.value)}
                placeholder="输入微信号"
              />
            </div>
          </div>
        </div>

        <div className="flex justify-end gap-4">
          <Button type="button" variant="outline" onClick={() => navigate(-1)}>
            取消
          </Button>
          <Button type="submit" disabled={isLoading}>
            <Save className="mr-2 h-4 w-4" />
            保存
          </Button>
        </div>
      </form>
    </div>
  )
}
