import {
  CircleCheckIcon,
  InfoIcon,
  Loader2Icon,
  OctagonXIcon,
  TriangleAlertIcon,
} from "lucide-react"
import { Toaster as Sonner, type ToasterProps } from "sonner"
import { isMobileDevice } from "@/lib/utils"

const Toaster = ({ ...props }: ToasterProps) => {
  const isMobile = isMobileDevice()

  return (
    <Sonner
      theme="system"
      className="toaster group"
      position={isMobile ? "bottom-center" : "top-center"}
      icons={{
        success: <CircleCheckIcon className="size-4" />,
        info: <InfoIcon className="size-4" />,
        warning: <TriangleAlertIcon className="size-4" />,
        error: <OctagonXIcon className="size-4" />,
        loading: <Loader2Icon className="size-4 animate-spin" />,
      }}
      style={
        {
          "--normal-bg": "var(--popover)",
          "--normal-text": "var(--popover-foreground)",
          "--normal-border": "var(--border)",
          "--border-radius": "var(--radius)",
          // 移动端额外样式 - 避免被底部导航栏遮挡
          ...(isMobile && {
            "--offset": "env(safe-area-inset-bottom, 16px)",
          } as React.CSSProperties),
        } as React.CSSProperties
      }
      {...props}
    />
  )
}

export { Toaster }
