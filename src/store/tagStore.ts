import { create } from 'zustand'
import * as tagApi from '@/services/tagApi'

export interface Tag {
  id: string
  name: string
  color: string | null
  createdAt: number
  updatedAt: number
}

export interface CreateTagRequest {
  name: string
  color?: string
}

export interface UpdateTagRequest {
  name?: string
  color?: string
}

interface TagState {
  tags: Tag[]
  isLoading: boolean
  error: string | null

  // 加载所有标签
  loadTags: () => Promise<void>

  // 创建标签
  createTag: (req: CreateTagRequest) => Promise<Tag>

  // 更新标签
  updateTag: (id: string, req: UpdateTagRequest) => Promise<Tag>

  // 删除标签
  deleteTag: (id: string) => Promise<void>

  // 获取笔记标签
  getNoteTags: (noteId: string) => Promise<Tag[]>

  // 添加标签到笔记
  addTagToNote: (noteId: string, tagId: string) => Promise<void>

  // 从笔记移除标签
  removeTagFromNote: (noteId: string, tagId: string) => Promise<void>

  // 设置笔记标签
  setNoteTags: (noteId: string, tagIds: string[]) => Promise<void>
}

export const useTagStore = create<TagState>((set, get) => ({
  tags: [],
  isLoading: false,
  error: null,

  loadTags: async () => {
    set({ isLoading: true, error: null })
    try {
      const tags = await tagApi.getAllTags()
      set({ tags, isLoading: false })
    } catch (error) {
      set({ error: (error as Error).message, isLoading: false })
    }
  },

  createTag: async (req) => {
    set({ isLoading: true, error: null })
    try {
      const newTag = await tagApi.createTag(req)
      set(state => ({
        tags: [...state.tags, newTag],
        isLoading: false,
      }))
      return newTag
    } catch (error) {
      set({ error: (error as Error).message, isLoading: false })
      throw error
    }
  },

  updateTag: async (id, req) => {
    set({ isLoading: true, error: null })
    try {
      const updatedTag = await tagApi.updateTag(id, req)
      set(state => ({
        tags: state.tags.map(tag =>
          tag.id === id ? updatedTag : tag
        ),
        isLoading: false,
      }))
      return updatedTag
    } catch (error) {
      set({ error: (error as Error).message, isLoading: false })
      throw error
    }
  },

  deleteTag: async (id) => {
    set({ isLoading: true, error: null })
    try {
      await tagApi.deleteTag(id)
      set(state => ({
        tags: state.tags.filter(tag => tag.id !== id),
        isLoading: false,
      }))
    } catch (error) {
      set({ error: (error as Error).message, isLoading: false })
      throw error
    }
  },

  getNoteTags: async (noteId) => {
    try {
      return await tagApi.getNoteTags(noteId)
    } catch (error) {
      set({ error: (error as Error).message })
      throw error
    }
  },

  addTagToNote: async (noteId, tagId) => {
    try {
      await tagApi.addTagToNote({ noteId, tagId })
    } catch (error) {
      set({ error: (error as Error).message })
      throw error
    }
  },

  removeTagFromNote: async (noteId, tagId) => {
    try {
      await tagApi.removeTagFromNote(noteId, tagId)
    } catch (error) {
      set({ error: (error as Error).message })
      throw error
    }
  },

  setNoteTags: async (noteId, tagIds) => {
    try {
      await tagApi.setNoteTags(noteId, tagIds)
    } catch (error) {
      set({ error: (error as Error).message })
      throw error
    }
  },
}))
