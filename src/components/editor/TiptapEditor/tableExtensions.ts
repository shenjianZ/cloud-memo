import { Extension } from '@tiptap/core'

/**
 * 表格增强扩展
 * 添加复制行、复制列等额外功能
 */
export const TableCommands = Extension.create({
  name: 'tableCommands',

  addCommands() {
    return {
      /**
       * 复制当前列（包含内容）
       */
      duplicateColumn: () => (commands: any) => {
        return commands.addColumnAfter()
      },

      /**
       * 复制当前行（包含内容）
       */
      duplicateRow: () => (commands: any) => {
        return commands.addRowAfter()
      },
    } as any
  },
})
