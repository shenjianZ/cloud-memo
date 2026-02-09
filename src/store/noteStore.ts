import { create } from "zustand";
import { persist } from "zustand/middleware";
import * as noteApi from "@/services/noteApi";
import * as folderApi from "@/services/folderApi";
import { useTagStore } from "./tagStore";
import type {
    Note as ApiNote,
    CreateNoteRequest,
    UpdateNoteRequest,
} from "@/services/noteApi";
import type { UpdateFolderRequest } from "@/types/folder";
import type { Note, NoteFolder, NoteFilter } from "@/types/note";
import { save } from "@tauri-apps/plugin-dialog";
import { writeTextFile } from "@tauri-apps/plugin-fs";
import { tiptapJsonToMarkdown } from "@/lib/tiptapSerializer";
import { getNoteTitle } from "@/lib/noteHelpers";
import { getNewFolderDepth, getDefaultFolderColor } from "@/lib/folderHelpers";
import { toast } from "sonner";

/**
 * 将 API Note 类型转换为应用 Note 类型
 * 支持 Tiptap JSON 和 Markdown 字符串
 */
async function apiNoteToNote(apiNote: ApiNote, tagStore?: any): Promise<Note> {
    // 尝试解析 content 为 Tiptap JSON，如果是旧 Markdown 格式则保持字符串
    let content: Note["content"];
    try {
        const parsed = JSON.parse(apiNote.content);
        // 验证是否为有效的 Tiptap JSON 结构
        if (parsed && typeof parsed === "object" && parsed.type === "doc") {
            content = parsed;
        } else {
            content = apiNote.content; // 保持 Markdown 字符串
        }
    } catch {
        // JSON 解析失败，保持 Markdown 字符串格式
        content = apiNote.content;
    }

    // 加载标签
    let tags: string[] = [];
    try {
        if (tagStore) {
            const noteTags = await tagStore.getNoteTags(apiNote.id);
            tags = noteTags.map((t: any) => t.id);
        }
    } catch (error) {
        console.error("Failed to load tags:", error);
    }

    return {
        id: apiNote.id,
        title: apiNote.title,
        author: apiNote.author,
        content,
        markdownCache: apiNote.markdownCache,
        createdAt: apiNote.createdAt * 1000, // 转换秒级时间戳为毫秒
        updatedAt: apiNote.updatedAt * 1000, // 转换秒级时间戳为毫秒
        tags,
        folder: apiNote.folderId,
        isPinned: apiNote.isPinned,
        isFavorite: apiNote.isFavorite,
    };
}

/**
 * 将应用 Note 类型转换为 API CreateNoteRequest
 * 将 Tiptap JSON 对象序列化为字符串
 */
function noteToCreateRequest(note: Partial<Note>): CreateNoteRequest {
    // 处理 content：如果是对象则序列化为 JSON 字符串
    const content =
        typeof note.content === "object"
            ? JSON.stringify(note.content)
            : note.content || "";

    return {
        title: note.title || "未命名笔记",
        content,
        folderId: note.folder,
    };
}

/**
 * 将应用 Note 类型转换为 API UpdateNoteRequest
 * 将 Tiptap JSON 对象序列化为字符串
 */
function noteToUpdateRequest(
    id: string,
    updates: Partial<Note>,
): UpdateNoteRequest {
    // 处理 content：如果是对象则序列化为 JSON 字符串
    const content =
        updates.content !== undefined
            ? typeof updates.content === "object"
                ? JSON.stringify(updates.content)
                : updates.content
            : undefined;

    return {
        id,
        title: updates.title,
        content,
        folderId: updates.folder,
        isFavorite: updates.isFavorite,
        isPinned: updates.isPinned,
        author: updates.author,
    };
}

interface NoteStore {
    notes: Note[];
    folders: NoteFolder[];
    activeNoteId: string | null;
    isLoading: boolean;
    isStorageLoaded: boolean;
    notesCount: number; // 笔记总数（不包括软删除的）

    // 笔记操作
    createNote: (note: Partial<Note>) => Promise<Note>;
    updateNote: (id: string, updates: Partial<Note>) => Promise<void>;
    deleteNote: (id: string) => Promise<void>;
    restoreNote: (id: string) => Promise<void>; // 恢复单个笔记
    restoreNotes: (ids: string[]) => Promise<void>; // 批量恢复笔记
    duplicateNote: (id: string) => Promise<Note>;
    exportNote: (id: string) => Promise<void>;

    // 标签操作
    setNoteTags: (noteId: string, tagIds: string[]) => Promise<void>;

    // 文件夹操作
    createFolder: (name: string, parentId?: string) => Promise<NoteFolder>;
    updateFolder: (id: string, updates: Partial<NoteFolder>) => Promise<void>;
    deleteFolder: (id: string) => Promise<void>;
    moveFolder: (id: string, newParentId: string | null) => Promise<void>;
    setFolderColor: (id: string, color: string) => Promise<void>;

    // 笔记移动
    moveNote: (noteId: string, folderId: string | null) => Promise<void>;
    moveNotes: (noteIds: string[], folderId: string | null) => Promise<void>;

    // 查询
    getNote: (id: string) => Note | undefined;
    searchNotes: (filter: NoteFilter) => Note[];
    searchNotesApi: (query: string) => Promise<Note[]>;

    // 文件夹辅助函数
    getNoteIdsInFolder: (folderId: string) => string[];

    // 存储
    loadNotesFromStorage: () => Promise<void>;
    saveNotesToStorage: () => Promise<void>;

    // 统计
    refreshNotesCount: () => Promise<void>;

    // 批量操作
    pinNote: (id: string) => void;
    favoriteNote: (id: string) => void;

    // 数据管理
    exportAllNotes: () => Promise<void>;
    clearAllNotes: () => Promise<void>;
    clearNotesState: () => void;  // 新增：只清空前端状态，不删除数据库记录
}

export const useNoteStore = create<NoteStore>()(
    persist(
        (set, get) => ({
            notes: [],
            folders: [],
            activeNoteId: null,
            isLoading: false,
            isStorageLoaded: false,
            notesCount: 0,

            refreshNotesCount: async () => {
                try {
                    const count = await noteApi.getNotesCount();
                    set({ notesCount: count });
                } catch (error) {
                    console.error("Failed to refresh notes count:", error);
                }
            },

            createNote: async (note: Partial<Note>) => {
                set({ isLoading: true });
                try {
                    const request = noteToCreateRequest(note);
                    const apiNote = await noteApi.createNote(request);
                    const tagStore = useTagStore.getState();
                    const newNote = await apiNoteToNote(apiNote, tagStore);

                    set((state) => ({
                        notes: [...state.notes, newNote],
                        activeNoteId: newNote.id,
                        isLoading: false,
                    }));

                    // 刷新笔记数量
                    get().refreshNotesCount();

                    return newNote;
                } catch (error) {
                    console.error("Failed to create note:", error);
                    set({ isLoading: false });
                    throw error;
                }
            },

            updateNote: async (id, updates) => {
                set({ isLoading: true });
                try {
                    const request = noteToUpdateRequest(id, updates);
                    const apiNote = await noteApi.updateNote(request);
                    const tagStore = useTagStore.getState();
                    const updatedNote = await apiNoteToNote(apiNote, tagStore);

                    set((state) => ({
                        notes: state.notes.map((n) =>
                            n.id === id ? updatedNote : n,
                        ),
                        isLoading: false,
                    }));
                } catch (error) {
                    console.error("Failed to update note:", error);
                    set({ isLoading: false });
                    throw error;
                }
            },

            deleteNote: async (id) => {
                set({ isLoading: true });
                try {
                    await noteApi.deleteNote(id);

                    set((state) => ({
                        notes: state.notes.filter((n) => n.id !== id),
                        activeNoteId:
                            state.activeNoteId === id
                                ? null
                                : state.activeNoteId,
                        isLoading: false,
                    }));

                    // 刷新笔记数量
                    get().refreshNotesCount();
                } catch (error) {
                    console.error("Failed to delete note:", error);
                    set({ isLoading: false });
                    throw error;
                }
            },

            restoreNote: async (id) => {
                set({ isLoading: true });
                try {
                    await noteApi.restoreNote(id);

                    // 重新加载所有笔记和文件夹（确保显示恢复的"已恢复笔记"文件夹）
                    await get().loadNotesFromStorage();

                    // 刷新笔记数量
                    get().refreshNotesCount();

                    // 只刷新列表，不跳转页面
                    toast.success("笔记已恢复", {
                        description: "已添加到笔记列表",
                    });
                } catch (error) {
                    console.error("Failed to restore note:", error);
                    set({ isLoading: false });
                    toast.error("恢复失败", {
                        description:
                            error instanceof Error ? error.message : "未知错误",
                    });
                    throw error;
                }
            },

            restoreNotes: async (ids) => {
                set({ isLoading: true });
                try {
                    const apiNotes = await noteApi.restoreNotes(ids);

                    // 重新加载所有笔记和文件夹（确保显示恢复的"已恢复笔记"文件夹）
                    await get().loadNotesFromStorage();

                    // 刷新笔记数量
                    get().refreshNotesCount();

                    // 只刷新列表，不跳转页面
                    toast.success(`已恢复 ${apiNotes.length} 篇笔记`, {
                        description: "已添加到笔记列表",
                    });
                } catch (error) {
                    console.error("Failed to restore notes:", error);
                    set({ isLoading: false });
                    toast.error("批量恢复失败", {
                        description:
                            error instanceof Error ? error.message : "未知错误",
                    });
                    throw error;
                }
            },

            duplicateNote: async (id) => {
                const original = get().notes.find((n) => n.id === id);
                if (!original) throw new Error("Note not found");

                return get().createNote({
                    ...original,
                    title: `${original.title} (副本)`,
                });
            },

            exportNote: async (id) => {
                const note = get().notes.find((n) => n.id === id);
                if (!note) throw new Error("Note not found");

                try {
                    const markdown = tiptapJsonToMarkdown(note.content);
                    const title = getNoteTitle(note);
                    const blob = new Blob([markdown], {
                        type: "text/markdown",
                    });
                    const url = URL.createObjectURL(blob);
                    const a = document.createElement("a");
                    a.href = url;
                    a.download = `${title}.md`;
                    document.body.appendChild(a);
                    a.click();
                    document.body.removeChild(a);
                    URL.revokeObjectURL(url);
                    toast.success("导出成功");
                } catch (error) {
                    console.error("Failed to export note:", error);
                    toast.error("导出失败");
                    throw error;
                }
            },

            createFolder: async (name, parentId) => {
                set({ isLoading: true });
                try {
                    // 计算新文件夹的层级并获取默认颜色
                    const depth = getNewFolderDepth(parentId || null, get().folders);
                    const defaultColor = getDefaultFolderColor(depth);

                    const apiFolder = await folderApi.createFolder({
                        name,
                        parentId,
                        color: defaultColor, // 设置默认颜色
                    });
                    // 将后端 Folder 类型转换为前端 NoteFolder 类型
                    const folder: NoteFolder = {
                        id: apiFolder.id,
                        name: apiFolder.name,
                        parentId: apiFolder.parentId || null,
                        color: apiFolder.color,
                        icon: apiFolder.icon,
                        sortOrder: apiFolder.sortOrder,
                        createdAt: apiFolder.createdAt * 1000, // 转换秒级时间戳为毫秒
                    };
                    set((state) => ({
                        folders: [...state.folders, folder],
                        isLoading: false,
                    }));
                    return folder;
                } catch (error) {
                    console.error("Failed to create folder:", error);
                    set({ isLoading: false });
                    throw error;
                }
            },

            deleteFolder: async (id) => {
                set({ isLoading: true });
                try {
                    await folderApi.deleteFolder(id);
                    set((state) => ({
                        folders: state.folders.filter((f) => f.id !== id),
                        isLoading: false,
                    }));

                    // 刷新笔记数量（删除文件夹可能会删除其中的笔记）
                    get().refreshNotesCount();
                } catch (error) {
                    console.error("Failed to delete folder:", error);
                    set({ isLoading: false });
                    throw error;
                }
            },

            updateFolder: async (id, updates) => {
                set({ isLoading: true });
                try {
                    // 构建 UpdateFolderRequest，只包含定义的字段
                    const req: UpdateFolderRequest = { id };
                    if (updates.name !== undefined) req.name = updates.name;
                    if (updates.parentId !== undefined)
                        req.parentId = updates.parentId || undefined;
                    if (updates.color !== undefined) req.color = updates.color;
                    if (updates.icon !== undefined) req.icon = updates.icon;
                    if (updates.sortOrder !== undefined)
                        req.sortOrder = updates.sortOrder;

                    await folderApi.updateFolder(req);
                    set((state) => ({
                        folders: state.folders.map((f) =>
                            f.id === id ? { ...f, ...updates } : f,
                        ),
                        isLoading: false,
                    }));
                } catch (error) {
                    console.error("Failed to update folder:", error);
                    set({ isLoading: false });
                    throw error;
                }
            },

            moveFolder: async (id, newParentId) => {
                set({ isLoading: true });
                try {
                    await folderApi.moveFolder({
                        id,
                        newParentId: newParentId || undefined,
                    });
                    set((state) => ({
                        folders: state.folders.map((f) =>
                            f.id === id ? { ...f, parentId: newParentId } : f,
                        ),
                        isLoading: false,
                    }));
                } catch (error) {
                    console.error("Failed to move folder:", error);
                    set({ isLoading: false });
                    throw error;
                }
            },

            setFolderColor: async (id, color) => {
                set({ isLoading: true });
                try {
                    await folderApi.updateFolder({ id, color });
                    set((state) => ({
                        folders: state.folders.map((f) =>
                            f.id === id ? { ...f, color } : f,
                        ),
                        isLoading: false,
                    }));
                } catch (error) {
                    console.error("Failed to set folder color:", error);
                    set({ isLoading: false });
                    throw error;
                }
            },

            moveNote: async (noteId, folderId) => {
                set({ isLoading: true });
                try {
                    await noteApi.updateNote({
                        id: noteId,
                        folderId: folderId || undefined,
                    });
                    set((state) => ({
                        notes: state.notes.map((n) =>
                            n.id === noteId
                                ? { ...n, folder: folderId || undefined }
                                : n,
                        ),
                        isLoading: false,
                    }));
                } catch (error) {
                    console.error("Failed to move note:", error);
                    set({ isLoading: false });
                    throw error;
                }
            },

            moveNotes: async (noteIds, folderId) => {
                set({ isLoading: true });
                try {
                    await folderApi.moveNotesToFolder({
                        noteIds,
                        folderId: folderId || undefined,
                    });
                    set((state) => ({
                        notes: state.notes.map((n) =>
                            noteIds.includes(n.id)
                                ? { ...n, folder: folderId || undefined }
                                : n,
                        ),
                        isLoading: false,
                    }));
                } catch (error) {
                    console.error("Failed to move notes:", error);
                    set({ isLoading: false });
                    throw error;
                }
            },

            getNote: (id) => {
                return get().notes.find((n) => n.id === id);
            },

            searchNotes: (filter) => {
                const { notes } = get();
                return notes.filter((note) => {
                    // 标题搜索
                    if (filter.query) {
                        const title = getNoteTitle(note).toLowerCase();
                        if (!title.includes(filter.query.toLowerCase())) {
                            return false;
                        }
                    }

                    // 标签过滤
                    if (filter.tags.length > 0) {
                        const hasAllTags = filter.tags.every((tag) =>
                            note.tags.includes(tag),
                        );
                        if (!hasAllTags) return false;
                    }

                    // 文件夹过滤
                    if (filter.folder && note.folder !== filter.folder) {
                        return false;
                    }

                    // 收藏过滤
                    if (filter.favorites && !note.isFavorite) {
                        return false;
                    }

                    return true;
                });
            },

            searchNotesApi: async (query) => {
                try {
                    const apiNotes = await noteApi.searchNotes(query);
                    const tagStore = useTagStore.getState();
                    const notes = await Promise.all(
                        apiNotes.map((apiNote) =>
                            apiNoteToNote(apiNote, tagStore),
                        ),
                    );
                    return notes;
                } catch (error) {
                    console.error("Failed to search notes:", error);
                    return [];
                }
            },

            setNoteTags: async (noteId, tagIds) => {
                try {
                    const tagStore = useTagStore.getState();
                    await tagStore.setNoteTags(noteId, tagIds);

                    // 更新本地笔记的标签
                    set((state) => ({
                        notes: state.notes.map((n) =>
                            n.id === noteId ? { ...n, tags: tagIds } : n,
                        ),
                    }));
                } catch (error) {
                    console.error("Failed to set note tags:", error);
                    throw error;
                }
            },

            loadNotesFromStorage: async () => {
                set({ isLoading: true });
                try {
                    // 加载笔记
                    const apiNotes = await noteApi.listNotes();
                    const tagStore = useTagStore.getState();
                    const notes = await Promise.all(
                        apiNotes.map((apiNote) =>
                            apiNoteToNote(apiNote, tagStore),
                        ),
                    );

                    // 加载文件夹
                    const apiFolders = await folderApi.listFolders();
                    const folders: NoteFolder[] = apiFolders.map(
                        (apiFolder) => ({
                            id: apiFolder.id,
                            name: apiFolder.name,
                            parentId: apiFolder.parentId || null,
                            color: apiFolder.color,
                            icon: apiFolder.icon,
                            sortOrder: apiFolder.sortOrder,
                            createdAt: apiFolder.createdAt * 1000, // 转换秒级时间戳为毫秒
                        }),
                    );

                    set({
                        notes,
                        folders,
                        isLoading: false,
                        isStorageLoaded: true,
                    });

                    // 刷新笔记数量
                    get().refreshNotesCount();
                } catch (error) {
                    console.error("Failed to load notes from storage:", error);
                    set({ isLoading: false, isStorageLoaded: true });
                }
            },

            saveNotesToStorage: async () => {
                // Notes are saved individually in updateNote/createNote
                // This method can be used for batch save if needed
            },

            pinNote: (id) => {
                const note = get().notes.find((n) => n.id === id);
                if (note) {
                    get().updateNote(id, { isPinned: !note.isPinned });
                }
            },

            favoriteNote: (id) => {
                const note = get().notes.find((n) => n.id === id);
                if (note) {
                    get().updateNote(id, { isFavorite: !note.isFavorite });
                }
            },

            exportAllNotes: async () => {
                try {
                    const { notes } = get();

                    // 创建包含所有笔记的 Markdown 内容
                    let markdownContent = `# 笔记备份\n\n`;
                    markdownContent += `导出时间: ${new Date().toLocaleString()}\n\n`;
                    markdownContent += `---\n\n`;

                    for (const note of notes) {
                        const title = getNoteTitle(note);
                        markdownContent += `# ${title}\n\n`;
                        markdownContent += tiptapJsonToMarkdown(note.content);
                        markdownContent += `\n\n---\n\n`;
                    }

                    // 打开保存对话框
                    const filePath = await save({
                        filters: [
                            {
                                name: "Markdown",
                                extensions: ["md"],
                            },
                        ],
                        defaultPath: `notes_backup_${Date.now()}.md`,
                    });

                    if (filePath) {
                        await writeTextFile(filePath, markdownContent);
                        toast.success("导出成功", {
                            description: `已导出 ${notes.length} 篇笔记`,
                        });
                    }
                } catch (error) {
                    console.error("Failed to export notes:", error);
                    toast.error("导出失败", {
                        description:
                            error instanceof Error ? error.message : "未知错误",
                    });
                }
            },

            clearAllNotes: async () => {
                try {
                    const { notes } = get();

                    // 删除所有笔记
                    for (const note of notes) {
                        await noteApi.deleteNote(note.id);
                    }

                    set({ notes: [], activeNoteId: null });
                    toast.success("所有数据已清除");
                } catch (error) {
                    console.error("Failed to clear notes:", error);
                    toast.error("清除失败", {
                        description:
                            error instanceof Error ? error.message : "未知错误",
                    });
                    throw error;
                }
            },

            // 新增：只清空前端状态，不删除数据库记录（用于切换账号）
            clearNotesState: () => {
                set({ notes: [], activeNoteId: null, folders: [] });
            },

            // 获取文件夹下的所有笔记 ID（包括子文件夹）
            getNoteIdsInFolder: (folderId: string) => {
                const { notes, folders } = get();
                const noteIds: string[] = [];

                // 递归获取子文件夹 ID
                const getSubfolderIds = (parentId: string): string[] => {
                    const subfolders = folders
                        .filter((f) => f.parentId === parentId)
                        .map((f) => f.id);
                    return subfolders.concat(
                        ...subfolders.flatMap((id) => getSubfolderIds(id))
                    );
                };

                // 获取所有子文件夹 ID（包括自身）
                const allFolderIds = [folderId, ...getSubfolderIds(folderId)];

                // 获取这些文件夹下的所有笔记 ID
                for (const note of notes) {
                    if (note.folder && allFolderIds.includes(note.folder)) {
                        noteIds.push(note.id);
                    }
                }

                return noteIds;
            },
        }),
        {
            name: "markdown-notes-storage",
            partialize: (state) => ({
                //Folders are loaded from database, not persisted in localStorage
                activeNoteId: state.activeNoteId,
            }),
        },
    ),
);
