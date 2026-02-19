import { useState, useEffect } from 'react'
import { TauriAPI } from '@/lib/tauri-api'
import type { StorageInfo, FileInfo } from '@/lib/tauri-api'
import { formatBytes, formatDate, showNotification } from '@/lib/utils'

export function FileBrowser() {
  const [storageInfo, setStorageInfo] = useState<StorageInfo | null>(null)
  const [files, setFiles] = useState<FileInfo[]>([])
  const [currentFolder, setCurrentFolder] = useState<string | null>(null)
  const [showModal, setShowModal] = useState(false)
  const [loading, setLoading] = useState(true)

  useEffect(() => {
    TauriAPI.getStorageInfo().then(setStorageInfo).catch(console.error).finally(() => setLoading(false))
  }, [])

  async function loadFiles(folder: string) {
    setCurrentFolder(folder)
    try {
      const list = await TauriAPI.listFiles(folder)
      setFiles(list)
    } catch (err) {
      showNotification('加载文件失败: ' + err, 'error')
    }
  }

  async function handleOpenFolder() {
    if (!storageInfo || !currentFolder) return
    try {
      await TauriAPI.openFolder(storageInfo.root_path + '/' + currentFolder)
    } catch (err) {
      showNotification('打开文件夹失败: ' + err, 'error')
    }
  }

  async function handleCleanup() {
    if (!currentFolder) return
    try {
      const result = await TauriAPI.cleanupOldFiles(30)
      showNotification(`已删除 ${result.deleted_count} 个文件，释放 ${formatBytes(result.freed_bytes)}`, 'success')
      setShowModal(false)
      loadFiles(currentFolder)
    } catch (err) {
      showNotification('清理失败: ' + err, 'error')
    }
  }

  const usagePercent = storageInfo
    ? Math.min((storageInfo.total_used_bytes / (10 * 1024 ** 3)) * 100, 100)
    : 0

  const folderTabs = ['Screenshots', 'Memories', 'Logs']

  return (
    <div className="w-screen h-screen bg-page overflow-y-auto custom-scrollbar">
      <div className="max-w-[1200px] mx-auto p-10 space-y-8">
        <div className="flex items-center justify-between">
          <h1 className="text-white text-[28px] font-bold">File Management</h1>
          <button
            onClick={() => window.location.href = '/'}
            className="px-6 py-2 bg-card border border-primary rounded-[12px] text-white font-medium hover:border-glow transition-colors cursor-pointer"
          >Back</button>
        </div>

        {/* Storage Overview */}
        <div className="bg-card border border-primary rounded-[20px] p-7">
          <h2 className="text-white text-lg font-bold mb-6">Storage Overview</h2>
          {loading ? (
            <div className="text-muted text-center py-8">Loading storage information...</div>
          ) : storageInfo ? (
            <>
              <div className="mb-6">
                <div className="flex items-center justify-between mb-2">
                  <span className="text-secondary text-sm">Total Usage</span>
                  <span className="text-white font-mono">{formatBytes(storageInfo.total_used_bytes)}</span>
                </div>
                <div className="h-3 bg-input rounded-full overflow-hidden">
                  <div className="h-full gradient-primary rounded-full transition-all duration-500" style={{ width: `${usagePercent}%` }} />
                </div>
              </div>
              <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
                {[
                  { label: 'Screenshots', value: storageInfo.screenshots_bytes, color: 'text-info' },
                  { label: 'Memories', value: storageInfo.memories_bytes, color: 'text-success' },
                  { label: 'Database', value: storageInfo.database_bytes, color: 'text-warning' },
                  { label: 'Logs', value: storageInfo.logs_bytes, color: 'text-muted' },
                ].map(({ label, value, color }) => (
                  <div key={label} className="bg-input rounded-[12px] p-4 text-center">
                    <div className="text-2xl mb-2">{label}</div>
                    <div className={`${color} font-mono text-lg`}>{formatBytes(value)}</div>
                  </div>
                ))}
              </div>
              <div className="mt-6 flex items-center justify-between text-sm">
                <span className="text-muted">Total Files: <span className="text-white">{storageInfo.total_files}</span></span>
                <span className="text-muted">Location: <span className="text-tertiary font-mono text-xs">{storageInfo.root_path}</span></span>
              </div>
            </>
          ) : null}
        </div>

        {/* File Browser */}
        <div className="bg-card border border-primary rounded-[20px] p-7">
          <div className="flex items-center justify-between mb-6">
            <h2 className="text-white text-lg font-bold">File Browser</h2>
            <div className="flex gap-2">
              {folderTabs.map(folder => (
                <button
                  key={folder}
                  onClick={() => loadFiles(folder)}
                  className={`px-4 py-2 rounded-[10px] text-sm font-medium transition-all ${
                    currentFolder === folder ? 'gradient-primary text-white' : 'bg-input text-muted hover:text-white'
                  }`}
                >{folder}</button>
              ))}
            </div>
          </div>

          <div className="space-y-2 max-h-[400px] overflow-y-auto custom-scrollbar">
            {files.length === 0 ? (
              <div className="text-muted text-center py-8">
                {currentFolder ? 'No files found' : 'Select a folder to view files'}
              </div>
            ) : files.map(file => (
              <div key={file.path} className="flex items-center justify-between p-3 bg-input rounded-[10px] hover:bg-secondary transition-colors">
                <div>
                  <div className="text-sm text-primary font-mono">{file.name}</div>
                  <div className="text-xs text-muted mt-1">{formatDate(file.modified_at)}</div>
                </div>
                <span className="text-xs text-muted font-mono">{formatBytes(file.size_bytes)}</span>
              </div>
            ))}
          </div>

          <div className="flex gap-4 mt-6 pt-6 border-t border-primary">
            <button
              onClick={handleOpenFolder}
              disabled={!currentFolder}
              className="px-6 py-3 bg-input rounded-[12px] text-white font-medium hover:bg-secondary transition-colors cursor-pointer flex items-center gap-2 disabled:opacity-50"
            >Open in Finder</button>
            <button
              onClick={() => setShowModal(true)}
              disabled={!currentFolder}
              className="px-6 py-3 bg-input rounded-[12px] text-warning font-medium hover:bg-secondary transition-colors cursor-pointer flex items-center gap-2 disabled:opacity-50"
            >Cleanup Old Files (30+ days)</button>
          </div>
        </div>
      </div>

      {/* Cleanup Modal */}
      {showModal && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
          <div className="bg-card border border-primary rounded-[20px] p-8 max-w-md mx-4">
            <h3 className="text-white text-xl font-bold mb-4">Confirm Cleanup</h3>
            <p className="text-secondary mb-6">This will permanently delete all files older than 30 days. This action cannot be undone.</p>
            <div className="flex gap-4 justify-end">
              <button onClick={() => setShowModal(false)} className="px-6 py-3 bg-input rounded-[12px] text-white font-medium hover:bg-secondary transition-colors cursor-pointer">Cancel</button>
              <button onClick={handleCleanup} className="px-6 py-3 bg-red-600 rounded-[12px] text-white font-medium hover:bg-red-700 transition-colors cursor-pointer">Delete Files</button>
            </div>
          </div>
        </div>
      )}
    </div>
  )
}
