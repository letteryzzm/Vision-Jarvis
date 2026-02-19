export function showNotification(message: string, type: 'success' | 'error' | 'info') {
  const el = document.createElement('div')
  el.className = `fixed bottom-4 right-4 px-6 py-3 rounded-[12px] text-white font-medium z-50 ${
    type === 'success' ? 'bg-green-600' : type === 'error' ? 'bg-red-600' : 'bg-blue-600'
  }`
  el.textContent = message
  document.body.appendChild(el)
  setTimeout(() => el.remove(), 3000)
}

export function formatBytes(bytes: number): string {
  if (bytes === 0) return '0 B'
  const k = 1024
  const sizes = ['B', 'KB', 'MB', 'GB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i]
}

export function formatDate(timestamp: number): string {
  return new Date(timestamp * 1000).toLocaleString()
}
