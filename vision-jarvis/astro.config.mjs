// @ts-check
import { defineConfig } from 'astro/config';

// https://astro.build/config
export default defineConfig({
  // 输出静态站点（Tauri 需要）
  output: 'static',

  // 构建输出目录
  outDir: './dist',

  // 开发服务器配置
  server: {
    port: 4321,  // 与 tauri.conf.json 中的 devUrl 一致
    host: true,  // 允许网络访问（可选）
  },

  // Vite 配置（可选）
  vite: {
    // 清除控制台
    clearScreen: false,
    // 服务器配置
    server: {
      strictPort: true,  // 端口被占用时不自动尝试下一个
    },
    // 环境变量前缀（可选）
    envPrefix: ['VITE_', 'TAURI_'],
  },
});
