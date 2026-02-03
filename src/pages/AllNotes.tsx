export default function AllNotes() {
  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-3xl font-bold tracking-tight">所有笔记</h1>
        <p className="text-muted-foreground">
          查看和管理所有笔记
        </p>
      </div>

      <div className="rounded-lg border p-8 text-center text-muted-foreground">
        暂无笔记，按 Ctrl+N 创建新笔记
      </div>
    </div>
  )
}
