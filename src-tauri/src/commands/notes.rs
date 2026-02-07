use crate::services::NoteService;
use crate::models::{Note, CreateNoteRequest, UpdateNoteRequest, MoveNotesRequest};
use tauri::State;

/// Note service 类型别名
type NoteSvc<'a> = State<'a, NoteService>;

/// 创建笔记
#[tauri::command]
pub async fn create_note(
    req: CreateNoteRequest,
    service: NoteSvc<'_>,
) -> std::result::Result<Note, String> {
    log::info!("[commands/notes.rs::create_note] 创建笔记: title={}", req.title);

    service.create_note(req)
        .map_err(|e| {
            log::error!("[commands/notes.rs::create_note] 创建失败: {}", e);
            e.to_string()
        })
        .map(|note| {
            log::info!("[commands/notes.rs::create_note] 创建成功: id={}, title={}", note.id, note.title);
            note
        })
}

/// 获取单个笔记
#[tauri::command]
pub async fn get_note(
    id: String,
    service: NoteSvc<'_>,
) -> std::result::Result<Note, String> {
    log::debug!("[commands/notes.rs::get_note] 获取笔记: id={}", id);

    service.get_note_by_id(&id)
        .map_err(|e| {
            log::error!("[commands/notes.rs::get_note] 获取失败: id={}, error={}", id, e);
            e.to_string()
        })
}

/// 更新笔记
#[tauri::command]
pub async fn update_note(
    req: UpdateNoteRequest,
    service: NoteSvc<'_>,
) -> std::result::Result<Note, String> {
    let note_id = req.id.clone();
    let title_display = req.title.as_deref().unwrap_or("(未修改)");
    log::debug!("[commands/notes.rs::update_note] 更新笔记: id={}, title={}", note_id, title_display);

    service.update_note(req)
        .map_err(|e| {
            log::error!("[commands/notes.rs::update_note] 更新失败: id={}, error={}", note_id, e);
            e.to_string()
        })
        .map(|note| {
            log::debug!("[commands/notes.rs::update_note] 更新成功: id={}", note_id);
            note
        })
}

/// 删除笔记（软删除）
#[tauri::command]
pub async fn delete_note(
    id: String,
    service: NoteSvc<'_>,
) -> std::result::Result<(), String> {
    log::info!("[commands/notes.rs::delete_note] 删除笔记: id={}", id);

    service.delete_note(&id)
        .map_err(|e| {
            log::error!("[commands/notes.rs::delete_note] 删除失败: id={}, error={}", id, e);
            e.to_string()
        })
        .map(|_| {
            log::info!("[commands/notes.rs::delete_note] 删除成功: id={}", id);
        })
}

/// 恢复已删除的笔记到"已恢复笔记"文件夹
///
/// ## 前端调用示例
///
/// ```typescript
/// import { invoke } from '@tauri-apps/api/tauri';
///
/// // 恢复单个笔记
/// const note = await invoke('restore_note', { id: 'note-id' });
///
/// // 恢复后笔记会出现在"已恢复笔记"文件夹中
/// ```
#[tauri::command]
pub async fn restore_note(
    id: String,
    service: NoteSvc<'_>,
) -> std::result::Result<Note, String> {
    log::info!("[commands/notes.rs::restore_note] 恢复笔记: id={}", id);

    service.restore_note(&id)
        .map_err(|e| {
            log::error!("[commands/notes.rs::restore_note] 恢复失败: id={}, error={}", id, e);
            e.to_string()
        })
        .map(|note| {
            log::info!("[commands/notes.rs::restore_note] 恢复成功: id={}", note.id);
            note
        })
}

/// 批量恢复笔记到"已恢复笔记"文件夹
///
/// ## 前端调用示例
///
/// ```typescript
/// import { invoke } from '@tauri-apps/api/tauri';
///
/// // 批量恢复笔记
/// const notes = await invoke('restore_notes', {
///   noteIds: ['note-1', 'note-2', 'note-3']
/// });
///
/// // 返回成功恢复的笔记列表
/// console.log(`成功恢复 ${notes.length} 个笔记`);
/// ```
#[tauri::command]
pub async fn restore_notes(
    note_ids: Vec<String>,
    service: NoteSvc<'_>,
) -> std::result::Result<Vec<Note>, String> {
    log::info!("[commands/notes.rs::restore_notes] 批量恢复笔记: count={}", note_ids.len());

    service.restore_notes(note_ids)
        .map_err(|e| {
            log::error!("[commands/notes.rs::restore_notes] 批量恢复失败: {}", e);
            e.to_string()
        })
        .map(|notes| {
            log::info!("[commands/notes.rs::restore_notes] 批量恢复成功: count={}", notes.len());
            notes
        })
}

/// 获取所有笔记
#[tauri::command]
pub async fn list_notes(
    service: NoteSvc<'_>,
) -> std::result::Result<Vec<Note>, String> {
    log::debug!("[commands/notes.rs::list_notes] 获取笔记列表");

    service.list_all_notes()
        .map_err(|e| {
            log::error!("[commands/notes.rs::list_notes] 获取失败: {}", e);
            e.to_string()
        })
        .map(|notes| {
            log::debug!("[commands/notes.rs::list_notes] 获取成功: count={}", notes.len());
            notes
        })
}

/// 获取所有已删除的笔记（回收站）
///
/// ## 前端调用示例
///
/// ```typescript
/// import { invoke } from '@tauri-apps/api/tauri';
///
/// // 获取回收站笔记列表
/// const deletedNotes = await invoke<Note[]>('list_deleted_notes');
///
/// console.log(`回收站中有 ${deletedNotes.length} 篇笔记`);
/// ```
#[tauri::command]
pub async fn list_deleted_notes(
    service: NoteSvc<'_>,
) -> std::result::Result<Vec<Note>, String> {
    log::debug!("[commands/notes.rs::list_deleted_notes] 获取回收站笔记列表");

    service.list_deleted_notes()
        .map_err(|e| {
            log::error!("[commands/notes.rs::list_deleted_notes] 获取失败: {}", e);
            e.to_string()
        })
        .map(|notes| {
            log::debug!("[commands/notes.rs::list_deleted_notes] 获取成功: count={}", notes.len());
            notes
        })
}

/// 搜索笔记
#[tauri::command]
pub async fn search_notes(
    query: String,
    service: NoteSvc<'_>,
) -> std::result::Result<Vec<Note>, String> {
    log::debug!("[commands/notes.rs::search_notes] 搜索笔记: query={}", query);

    service.search_notes(&query)
        .map_err(|e| {
            log::error!("[commands/notes.rs::search_notes] 搜索失败: query={}, error={}", query, e);
            e.to_string()
        })
        .map(|notes| {
            log::debug!("[commands/notes.rs::search_notes] 搜索成功: query={}, count={}", query, notes.len());
            notes
        })
}

/// 批量移动笔记到文件夹
#[tauri::command]
pub async fn move_notes_to_folder(
    req: MoveNotesRequest,
    service: NoteSvc<'_>,
) -> std::result::Result<Vec<Note>, String> {
    let folder_id_display = req.folder_id.as_deref().unwrap_or("root");
    log::info!("[commands/notes.rs::move_notes_to_folder] 批量移动笔记: note_count={}, folder_id={}", req.note_ids.len(), folder_id_display);

    service.move_notes_to_folder(req)
        .map_err(|e| {
            log::error!("[commands/notes.rs::move_notes_to_folder] 移动失败: {}", e);
            e.to_string()
        })
        .map(|notes| {
            log::info!("[commands/notes.rs::move_notes_to_folder] 移动成功: count={}", notes.len());
            notes
        })
}

/// 永久删除笔记（硬删除）
#[tauri::command]
pub async fn permanently_delete_note(
    id: String,
    service: NoteSvc<'_>,
) -> std::result::Result<(), String> {
    log::info!("[commands/notes.rs::permanently_delete_note] 永久删除笔记: id={}", id);

    service.permanently_delete_note(&id)
        .map_err(|e| {
            log::error!("[commands/notes.rs::permanently_delete_note] 删除失败: id={}, error={}", id, e);
            e.to_string()
        })
        .map(|_| {
            log::info!("[commands/notes.rs::permanently_delete_note] 删除成功: id={}", id);
        })
}

/// 批量永久删除笔记
#[tauri::command]
pub async fn permanently_delete_notes(
    note_ids: Vec<String>,
    service: NoteSvc<'_>,
) -> std::result::Result<i64, String> {
    log::info!("[commands/notes.rs::permanently_delete_notes] 批量永久删除笔记: count={}", note_ids.len());

    service.permanently_delete_notes(note_ids)
        .map_err(|e| {
            log::error!("[commands/notes.rs::permanently_delete_notes] 批量删除失败: {}", e);
            e.to_string()
        })
        .map(|count| {
            log::info!("[commands/notes.rs::permanently_delete_notes] 批量删除成功: count={}", count);
            count
        })
}

/// 获取笔记数量（不包括软删除的笔记）
///
/// ## 前端调用示例
///
/// ```typescript
/// import { invoke } from '@tauri-apps/api/tauri';
///
/// // 获取笔记总数
/// const count = await invoke<number>('get_notes_count');
///
/// console.log(`当前共有 ${count} 篇笔记`);
/// ```
#[tauri::command]
pub async fn get_notes_count(
    service: NoteSvc<'_>,
) -> std::result::Result<i64, String> {
    log::debug!("[commands/notes.rs::get_notes_count] 获取笔记数量");

    service.count_notes()
        .map_err(|e| {
            log::error!("[commands/notes.rs::get_notes_count] 获取失败: {}", e);
            e.to_string()
        })
        .map(|count| {
            log::debug!("[commands/notes.rs::get_notes_count] 获取成功: count={}", count);
            count
        })
}
