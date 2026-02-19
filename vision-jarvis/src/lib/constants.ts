// 浮球窗口尺寸（像素）
export const WINDOW_SIZES = {
  ball:   { width: 64,  height: 64  },
  header: { width: 360, height: 146 },
  asker:  { width: 360, height: 554 },
} as const

// 动画与交互延迟（毫秒）
export const DELAYS = {
  transition:  150,
  hoverExpand: 200,
  hoverCollapse: 300,
  initHover:   500,
} as const
