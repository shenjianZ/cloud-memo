/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      // 移动端优先的断点配置
      screens: {
        'xs': '375px',   // 超小屏手机
        'sm': '640px',   // 小屏手机
        'md': '768px',   // 平板
        'lg': '1024px',  // 桌面
        'xl': '1280px',  // 大屏桌面
        '2xl': '1536px', // 超大屏
      },
      // 移动端友好的间距
      spacing: {
        'mobile-safe': 'env(safe-area-inset-bottom)',
      },
      // 移动端友好的圆角
      borderRadius: {
        'mobile': '12px',
      },
      // 移动端友好的阴影
      boxShadow: {
        'mobile': '0 2px 8px rgba(0, 0, 0, 0.1)',
      },
      // 移动端友好的动画
      transitionDuration: {
        '400': '400ms',
      },
    },
  },
  plugins: [],
}
