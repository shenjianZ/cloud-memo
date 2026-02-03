import { Outlet } from 'react-router-dom'
import { Sidebar } from './Sidebar'
import { TitleBar } from './TitleBar'

/**
 * 主布局组件 - 两层布局
 * 顶部：自定义标题栏
 * 左侧：合并的侧边栏（搜索、设置 + 文件夹树 + 笔记列表）
 * 右侧：编辑器区域
 */
export function MainLayout() {
  return (
    <>
      {/* 自定义标题栏 */}
      <TitleBar />

      {/* 主内容区域 - 带标题栏偏移 */}
      <div className="main-content-with-titlebar flex bg-background">
        {/* 左侧：合并的侧边栏 */}
        <Sidebar />

        {/* 右侧：编辑器区域 */}
        <div className="flex-1 flex flex-col overflow-hidden">
          <main className="flex-1 bg-background overflow-y-auto custom-scrollbar">
            <div className="h-full">
              <Outlet />
            </div>
          </main>
        </div>
      </div>
    </>
  )
}
