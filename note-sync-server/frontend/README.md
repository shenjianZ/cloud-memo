# Vite + React + TypeScript + shadcn 模板

这是一个预配置了 Vite, React, TypeScript 和 shadcn 的入门模板，可以帮助你快速启动新项目。

## ✨ 特性

- ⚡️ **Vite**: 极速的下一代前端构建工具。
- ⚛️ **React**: 用于构建用户界面的 JavaScript 库。
- 📘 **TypeScript**: JavaScript 的超集，添加了类型支持。
- 🎨 **Tailwind CSS**: 一个功能类优先的 CSS 框架。
- 🧩 **shadcn**: 设计精美、可重用的组件，你可以直接复制粘贴到你的应用中。

## 🚀 快速上手

1.  **克隆或使用此模板创建你的项目**

2.  **安装依赖**
    ```bash
    npm install
    ```

3.  **启动开发服务器**
    ```bash
    npm run dev
    ```
    现在，在浏览器中打开指定的本地地址 (通常是 `http://localhost:5173`) 即可查看。

## 📦 添加组件

现在你可以开始向你的项目添加组件了。

```bash
npx shadcn@latest add [component-name]
```

例如，要添加一个 `button` 组件：

```bash
npx shadcn@latest add button
```

新添加的组件会出现在 `src/components/ui` 目录下。你可以像这样导入它：

**`src/App.tsx`**
```tsx
import { Button } from "@/components/ui/button"

function App() {
  return (
    <div className="flex min-h-svh flex-col items-center justify-center">
      <Button>Click me</Button>
    </div>
  )
}

export default App
```

## 📜 可用脚本

在 `package.json` 中定义了以下脚本：

- `npm run dev`: 在开发模式下启动应用，支持热更新。
- `npm run build`: 为生产环境构建应用，输出到 `dist` 目录。
- `npm run lint`: 使用 ESLint 检查代码规范。
- `npm run preview`: 在本地预览生产构建的应用。

## 📄 许可证

本项目采用 MIT 许可证。