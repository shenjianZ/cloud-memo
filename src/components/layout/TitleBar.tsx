import { useState, useEffect } from 'react';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { Minus, Square, X, Copy } from 'lucide-react';
import './TitleBar.css';

// Windows 风格的还原图标（两个重叠的矩形）
// 说明：外层窗口用 path 绘制上边+右边，内层窗口是完整矩形
const RestoreIcon = ({ size = 18 }: { size?: number }) => (
  <svg
    width={size}
    height={size}
    viewBox="0 0 24 24"
    fill="none"
    xmlns="http://www.w3.org/2000/svg"
  >
    {/* 外层窗口（只画 上边 + 右边，圆角更大） */}
    <path
      d="M7 5 H18 A2.5 2.5 0 0 1 20.5 7.5 V18"
      stroke="currentColor"
      strokeWidth="1.6"
      strokeLinecap="round"
      strokeLinejoin="round"
    />

    {/* 内层窗口（完整矩形，圆角较小） */}
    <rect
      x="5"
      y="7.5"
      width="12"
      height="12"
      rx="1.6"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.6"
    />
  </svg>
);

const appWindow = getCurrentWindow();

export function TitleBar() {
  const [isMaximized, setIsMaximized] = useState(false);

  useEffect(() => {
    const checkMaximized = async () => {
      const maximized = await appWindow.isMaximized();
      setIsMaximized(maximized);
    };

    checkMaximized();

    // 监听窗口状态变化
    const unlisten = appWindow.onResized(() => {
      checkMaximized();
    });

    return () => {
      unlisten.then((fn) => fn?.());
    };
  }, []);

  const minimize = () => {
    appWindow.minimize();
  };

  const toggleMaximize = () => {
    if (isMaximized) {
      appWindow.unmaximize();
    } else {
      appWindow.maximize();
    }
  };

  const close = () => {
    appWindow.close();
  };

  return (
    <div className="titlebar">
      {/* 拖拽区域 - 左侧 */}
      <div data-tauri-drag-region className="titlebar-drag-area">
        <Copy className="titlebar-icon" size={16} />
        <span className="titlebar-title">Markdown Notes</span>
      </div>

      {/* 窗口控制按钮 - 右侧 */}
      <div className="window-controls">
        <button
          onClick={minimize}
          className="control-btn control-btn-minimize"
          title="最小化"
        >
          <Minus size={14} />
        </button>
        <button
          onClick={toggleMaximize}
          className="control-btn control-btn-maximize"
          title={isMaximized ? '还原' : '最大化'}
        >
          {isMaximized ? <RestoreIcon size={18} /> : <Square size={18} />}
        </button>
        <button
          onClick={close}
          className="control-btn control-btn-close"
          title="关闭"
        >
          <X size={14} />
        </button>
      </div>
    </div>
  );
}
