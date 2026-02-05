import { useAuthStore } from '@/store/authStore'
import { UserAvatarDropdown } from './UserAvatarDropdown'
import { User as UserIcon, Loader2 } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { openAuthWindow } from '@/lib/authWindow'

export function UserArea() {
  const { isAuthenticated, isLoading } = useAuthStore()

  const handleLoginClick = async () => {
    await openAuthWindow()
  }

  // 如果正在加载中，显示加载图标
  if (isLoading) {
    return (
      <div className="h-8 w-8 flex items-center justify-center">
        <Loader2 className="h-4 w-4 animate-spin text-muted-foreground" />
      </div>
    )
  }

  if (isAuthenticated) {
    return <UserAvatarDropdown />
  }

  return (
    <Button
      variant="ghost"
      size="icon"
      className="h-8 w-8"
      onClick={handleLoginClick}
      title="登录"
    >
      <UserIcon className="h-4 w-4" />
    </Button>
  )
}
