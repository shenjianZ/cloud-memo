import { LogOut, Settings, UserCircle, Users } from 'lucide-react'
import { useAuthStore } from '@/store/authStore'
import { useProfileStore } from '@/store/profileStore'
import { useNavigate } from 'react-router-dom'
import { useState } from 'react'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu'
import { Avatar, AvatarFallback, AvatarImage } from '@/components/ui/avatar'
import { AccountManager } from './AccountManager'

export function UserAvatarDropdown() {
  const { user, logout, allAccounts } = useAuthStore()
  const { profile } = useProfileStore()
  const navigate = useNavigate()
  const [accountManagerOpen, setAccountManagerOpen] = useState(false)

  const handleLogout = async () => {
    await logout()
  }

  const handleProfile = () => {
    navigate('/profile')
  }

  const handleSettings = () => {
    navigate('/settings')
  }

  // 生成用户名首字母作为头像 fallback
  const getInitials = (name: string) => {
    return name
      .split(' ')
      .map(n => n[0])
      .join('')
      .toUpperCase()
      .slice(0, 2)
  }

  // 获取头像显示名称
  const displayName = profile?.username || user?.email?.split('@')[0] || 'User'
  const initials = getInitials(displayName)

  // 获取头像源（从 UserProfile 中获取 Base64 数据）
  const getAvatarSrc = () => {
    if (profile?.avatarData && profile?.avatarMimeType) {
      return `data:${profile.avatarMimeType};base64,${profile.avatarData}`
    }
    return undefined
  }

  return (
    <>
      <DropdownMenu>
        <DropdownMenuTrigger asChild>
          <button className="h-8 w-8 rounded-full hover:bg-muted transition-colors flex items-center justify-center">
            <Avatar className="h-8 w-8">
              <AvatarImage src={getAvatarSrc()} />
              <AvatarFallback className="bg-primary text-primary-foreground text-xs font-medium">
                {initials}
              </AvatarFallback>
            </Avatar>
          </button>
        </DropdownMenuTrigger>
        <DropdownMenuContent align="end" className="w-56">
          <DropdownMenuLabel>
            <div className="flex flex-col space-y-1">
              <p className="text-sm font-medium leading-none">{displayName}</p>
              <p className="text-xs leading-none text-muted-foreground">
                {user?.email}
              </p>
            </div>
          </DropdownMenuLabel>
          <DropdownMenuSeparator />

          {/* 新增：账号列表快捷入口 */}
          {allAccounts.length > 1 && (
            <>
              <DropdownMenuItem onClick={() => setAccountManagerOpen(true)}>
                <Users className="mr-2 h-4 w-4" />
                <div className="flex-1">切换账号</div>
                <span className="text-xs text-muted-foreground ml-2">
                  {allAccounts.length}
                </span>
              </DropdownMenuItem>
              <DropdownMenuSeparator />
            </>
          )}

          {/* 即使只有一个账号，也显示账号管理选项 */}
          {allAccounts.length <= 1 && (
            <>
              <DropdownMenuItem onClick={() => setAccountManagerOpen(true)}>
                <Users className="mr-2 h-4 w-4" />
                <span>账号管理</span>
              </DropdownMenuItem>
              <DropdownMenuSeparator />
            </>
          )}

          <DropdownMenuItem onClick={handleProfile}>
            <UserCircle className="mr-2 h-4 w-4" />
            <span>个人中心</span>
          </DropdownMenuItem>
          <DropdownMenuItem onClick={handleSettings}>
            <Settings className="mr-2 h-4 w-4" />
            <span>设置</span>
          </DropdownMenuItem>
          <DropdownMenuSeparator />
          <DropdownMenuItem onClick={handleLogout}>
            <LogOut className="mr-2 h-4 w-4" />
            <span>登出</span>
          </DropdownMenuItem>
        </DropdownMenuContent>
      </DropdownMenu>

      {/* 账号管理对话框 */}
      <AccountManager
        open={accountManagerOpen}
        onOpenChange={setAccountManagerOpen}
      />
    </>
  )
}
