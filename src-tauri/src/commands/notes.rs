use crate::services::NoteService;
use crate::models::{Note, CreateNoteRequest, UpdateNoteRequest};
use tauri::State;

/// Note service 类型别名
type NoteSvc<'a> = State<'a, NoteService>;

/// 创建笔记
#[tauri::command]
pub async fn create_note(
    req: CreateNoteRequest,
    service: NoteSvc<'_>,
) -> std::result::Result<Note, String> {
    service.create_note(req)
        .map_err(|e| e.to_string())
}

/// 获取单个笔记
#[tauri::command]
pub async fn get_note(
    id: String,
    service: NoteSvc<'_>,
) -> std::result::Result<Note, String> {
    service.get_note_by_id(&id)
        .map_err(|e| e.to_string())
}

/// 更新笔记
#[tauri::command]
pub async fn update_note(
    req: UpdateNoteRequest,
    service: NoteSvc<'_>,
) -> std::result::Result<Note, String> {
    service.update_note(req)
        .map_err(|e| e.to_string())
}

/// 删除笔记（软删除）
#[tauri::command]
pub async fn delete_note(
    id: String,
    service: NoteSvc<'_>,
) -> std::result::Result<(), String> {
    service.delete_note(&id)
        .map_err(|e| e.to_string())
}

/// 获取所有笔记
#[tauri::command]
pub async fn list_notes(
    service: NoteSvc<'_>,
) -> std::result::Result<Vec<Note>, String> {
    service.list_all_notes()
        .map_err(|e| e.to_string())
}

/// 搜索笔记
#[tauri::command]
pub async fn search_notes(
    query: String,
    service: NoteSvc<'_>,
) -> std::result::Result<Vec<Note>, String> {
    service.search_notes(&query)
        .map_err(|e| e.to_string())
}
