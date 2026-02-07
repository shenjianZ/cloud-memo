import { create } from "zustand";
import { persist } from "zustand/middleware";
import * as syncApi from "@/services/syncApi";
import type { ConflictResolutionStrategy, SyncReport } from "@/types/sync";
import { toast } from "sonner";

export type SyncStatusType = "idle" | "syncing" | "error" | "conflict";

/**
 * 生成详细的同步统计消息（只显示不为0的项目）
 */
function formatSyncReportMessage(report: SyncReport): string {
    const parts: string[] = [];

    // 推送统计
    const pushedParts: string[] = [];
    if (report.pushedNotes > 0) pushedParts.push(`笔记 ${report.pushedNotes}`);
    if (report.pushedFolders > 0) pushedParts.push(`文件夹 ${report.pushedFolders}`);
    if (report.pushedTags > 0) pushedParts.push(`标签 ${report.pushedTags}`);
    if (report.pushedSnapshots > 0) pushedParts.push(`快照 ${report.pushedSnapshots}`);
    if (report.pushedNoteTags > 0) pushedParts.push(`标签关联 ${report.pushedNoteTags}`);

    // 拉取统计
    const pulledParts: string[] = [];
    if (report.pulledNotes > 0) pulledParts.push(`笔记 ${report.pulledNotes}`);
    if (report.pulledFolders > 0) pulledParts.push(`文件夹 ${report.pulledFolders}`);
    if (report.pulledTags > 0) pulledParts.push(`标签 ${report.pulledTags}`);
    if (report.pulledSnapshots > 0) pulledParts.push(`快照 ${report.pulledSnapshots}`);
    if (report.pulledNoteTags > 0) pulledParts.push(`标签关联 ${report.pulledNoteTags}`);

    // 删除统计
    const deletedParts: string[] = [];
    if (report.deletedNotes > 0) deletedParts.push(`笔记 ${report.deletedNotes}`);
    if (report.deletedFolders > 0) deletedParts.push(`文件夹 ${report.deletedFolders}`);
    if (report.deletedTags > 0) deletedParts.push(`标签 ${report.deletedTags}`);

    if (pushedParts.length > 0) {
        parts.push(`推送: ${pushedParts.join(", ")}`);
    }
    if (pulledParts.length > 0) {
        parts.push(`拉取: ${pulledParts.join(", ")}`);
    }
    if (deletedParts.length > 0) {
        parts.push(`删除: ${deletedParts.join(", ")}`);
    }

    if (report.conflictCount > 0) {
        parts.push(`冲突: ${report.conflictCount} 项`);
    }

    return parts.length > 0 ? parts.join("\n") : "已是最新";
}

interface SyncState {
    status: SyncStatusType;
    lastSyncAt: number | null;
    pendingCount: number;
    conflictCount: number;
    lastError: string | null;
    isAutoSyncEnabled: boolean;
    conflictResolution: ConflictResolutionStrategy;

    // Actions
    syncNow: (options?: { conflictResolution?: ConflictResolutionStrategy }) => Promise<void>;
    syncSingleNote: (noteId: string) => Promise<void>;
    syncSingleFolder: (folderId: string) => Promise<void>;
    syncSingleTag: (tagId: string) => Promise<void>;
    syncSingleSnapshot: (snapshotId: string) => Promise<void>;
    refreshStatus: () => Promise<void>;
    clearError: () => void;
    setAutoSync: (enabled: boolean) => void;
    setConflictResolution: (strategy: ConflictResolutionStrategy) => void;
}

export const useSyncStore = create<SyncState>()(
    persist(
        (set) => ({
            status: "idle",
            lastSyncAt: null,
            pendingCount: 0,
            conflictCount: 0,
            lastError: null,
            isAutoSyncEnabled: true,
            conflictResolution: "create_conflict_copy",

            syncNow: async (options) => {
                set({ status: "syncing", lastError: null });
                try {
                    // device_id 由 Tauri 后端自动管理
                    const report = await syncApi.syncNow({
                        conflictResolution: options?.conflictResolution || "create_conflict_copy",
                    });

                    if (report.success) {
                        set({
                            status: "idle",
                            lastSyncAt: Date.now(),
                            pendingCount: 0,
                            conflictCount: report.conflictCount,
                        });

                        // ✅ 显示详细的同步统计
                        const message = formatSyncReportMessage(report);
                        toast.success("同步成功", {
                            description: message,
                        });

                        // ✅ 刷新所有数据（使用动态导入避免循环依赖）
                        const { useNoteStore } = await import("@/store/noteStore");
                        const { useTagStore } = await import("@/store/tagStore");

                        await Promise.all([
                            useNoteStore.getState().loadNotesFromStorage(), // 包含笔记和文件夹
                            useTagStore.getState().loadTags(),
                        ]).catch((err) => {
                            console.error("Failed to refresh data after sync:", err);
                        });
                    } else {
                        set({
                            status: "error",
                            lastError: report.error || "同步失败",
                        });
                        toast.error(report.error || "同步失败");
                    }
                } catch (error) {
                    set({
                        status: "error",
                        lastError:
                            error instanceof Error ? error.message : "同步失败",
                    });
                    toast.error("同步失败");
                }
            },

            syncSingleNote: async (noteId: string) => {
                set({ status: "syncing", lastError: null });
                try {
                    // device_id 由 Tauri 后端自动管理
                    const report = await syncApi.syncSingleNote(noteId);

                    if (report.success) {
                        set({
                            status: "idle",
                            lastSyncAt: Date.now(),
                            // 减少待同步数量
                            pendingCount: Math.max(
                                0,
                                (useSyncStore.getState().pendingCount || 0) -
                                    report.conflictCount,
                            ),
                            conflictCount: report.conflictCount,
                        });

                        // ✅ 显示详细的同步统计
                        const message = formatSyncReportMessage(report);
                        toast.success("笔记同步成功", {
                            description: message,
                        });
                    } else {
                        set({
                            status: "error",
                            lastError: report.error || "同步失败",
                        });
                        toast.error(report.error || "同步失败");
                    }
                } catch (error) {
                    set({
                        status: "error",
                        lastError:
                            error instanceof Error ? error.message : "同步失败",
                    });
                    toast.error("同步失败");
                }
            },

            syncSingleFolder: async (folderId: string) => {
                set({ status: "syncing", lastError: null });
                try {
                    const report = await syncApi.syncSingleFolder(folderId);

                    if (report.success) {
                        set({
                            status: "idle",
                            lastSyncAt: Date.now(),
                            conflictCount: report.conflictCount,
                        });

                        // ✅ 显示详细的同步统计
                        const message = formatSyncReportMessage(report);
                        toast.success("文件夹同步成功", {
                            description: message,
                        });

                        // ✅ 刷新笔记列表（文件夹已包含在 loadNotesFromStorage 中）
                        const { useNoteStore } = await import("@/store/noteStore");

                        await useNoteStore.getState().loadNotesFromStorage()
                            .catch((err) => {
                                console.error("Failed to refresh data after sync:", err);
                            });
                    } else {
                        set({
                            status: "error",
                            lastError: report.error || "同步失败",
                        });
                        toast.error(report.error || "同步失败");
                    }
                } catch (error) {
                    set({
                        status: "error",
                        lastError:
                            error instanceof Error ? error.message : "同步失败",
                    });
                    toast.error("同步失败");
                }
            },

            syncSingleTag: async (tagId: string) => {
                set({ status: "syncing", lastError: null });
                try {
                    const report = await syncApi.syncSingleTag(tagId);

                    if (report.success) {
                        set({
                            status: "idle",
                            lastSyncAt: Date.now(),
                            conflictCount: report.conflictCount,
                        });

                        // ✅ 显示详细的同步统计
                        const message = formatSyncReportMessage(report);
                        toast.success("标签同步成功", {
                            description: message,
                        });

                        // ✅ 刷新标签列表
                        const { useTagStore } = await import("@/store/tagStore");
                        await useTagStore.getState().loadTags().catch((err) => {
                            console.error("Failed to refresh tags after sync:", err);
                        });
                    } else {
                        set({
                            status: "error",
                            lastError: report.error || "同步失败",
                        });
                        toast.error(report.error || "同步失败");
                    }
                } catch (error) {
                    set({
                        status: "error",
                        lastError:
                            error instanceof Error ? error.message : "同步失败",
                    });
                    toast.error("同步失败");
                }
            },

            syncSingleSnapshot: async (snapshotId: string) => {
                set({ status: "syncing", lastError: null });
                try {
                    const report = await syncApi.syncSingleSnapshot(snapshotId);

                    if (report.success) {
                        set({
                            status: "idle",
                            lastSyncAt: Date.now(),
                            conflictCount: report.conflictCount,
                        });

                        // ✅ 显示详细的同步统计
                        const message = formatSyncReportMessage(report);
                        toast.success("快照同步成功", {
                            description: message,
                        });
                    } else {
                        set({
                            status: "error",
                            lastError: report.error || "同步失败",
                        });
                        toast.error(report.error || "同步失败");
                    }
                } catch (error) {
                    set({
                        status: "error",
                        lastError:
                            error instanceof Error ? error.message : "同步失败",
                    });
                    toast.error("同步失败");
                }
            },

            refreshStatus: async () => {
                try {
                    const status = await syncApi.getSyncStatus();
                    set({
                        lastSyncAt: status.lastSyncAt,
                        pendingCount: status.pendingCount,
                        conflictCount: status.conflictCount,
                        lastError: status.lastError,
                    });
                } catch (error) {
                    console.error("Failed to refresh sync status:", error);
                }
            },

            clearError: () => set({ lastError: null, status: "idle" }),

            setAutoSync: (enabled: boolean) =>
                set({ isAutoSyncEnabled: enabled }),

            setConflictResolution: (strategy: ConflictResolutionStrategy) =>
                set({ conflictResolution: strategy }),
        }),
        {
            name: "sync-storage",
            partialize: (state) => ({
                lastSyncAt: state.lastSyncAt,
                isAutoSyncEnabled: state.isAutoSyncEnabled,
                conflictResolution: state.conflictResolution,
            }),
        },
    ),
);
