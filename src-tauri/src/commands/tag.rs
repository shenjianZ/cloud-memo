use crate::services::TagService;
use crate::models::{Tag, CreateTagRequest, UpdateTagRequest, NoteTagRequest};
use tauri::State;

type TagSvc<'a> = State<'a, TagService>;

/// 获取所有标签
#[tauri::command]
pub async fn get_all_tags(
    service: TagSvc<'_>,
) -> std::result::Result<Vec<Tag>, String> {
    service.get_all_tags().map_err(|e| e.to_string())
}

/// 根据 ID 获取标签
#[tauri::command]
pub async fn get_tag(
    id: String,
    service: TagSvc<'_>,
) -> std::result::Result<Tag, String> {
    service.get_tag(&id).map_err(|e| e.to_string())
}

/// 获取笔记的所有标签
#[tauri::command]
pub async fn get_note_tags(
    note_id: String,
    service: TagSvc<'_>,
) -> std::result::Result<Vec<Tag>, String> {
    service.get_note_tags(&note_id).map_err(|e| e.to_string())
}

/// 创建标签
#[tauri::command]
pub async fn create_tag(
    req: CreateTagRequest,
    service: TagSvc<'_>,
) -> std::result::Result<Tag, String> {
    service.create_tag(req).map_err(|e| e.to_string())
}

/// 更新标签
#[tauri::command]
pub async fn update_tag(
    id: String,
    req: UpdateTagRequest,
    service: TagSvc<'_>,
) -> std::result::Result<Tag, String> {
    service.update_tag(&id, req).map_err(|e| e.to_string())
}

/// 删除标签
#[tauri::command]
pub async fn delete_tag(
    id: String,
    service: TagSvc<'_>,
) -> std::result::Result<(), String> {
    service.delete_tag(&id).map_err(|e| e.to_string())
}

/// 为笔记添加标签
#[tauri::command]
pub async fn add_tag_to_note(
    req: NoteTagRequest,
    service: TagSvc<'_>,
) -> std::result::Result<(), String> {
    service.add_tag_to_note(req).map_err(|e| e.to_string())
}

/// 从笔记移除标签
#[tauri::command]
pub async fn remove_tag_from_note(
    note_id: String,
    tag_id: String,
    service: TagSvc<'_>,
) -> std::result::Result<(), String> {
    service.remove_tag_from_note(&note_id, &tag_id).map_err(|e| e.to_string())
}

/// 设置笔记的标签（替换所有标签）
#[tauri::command]
pub async fn set_note_tags(
    note_id: String,
    tag_ids: Vec<String>,
    service: TagSvc<'_>,
) -> std::result::Result<(), String> {
    service.set_note_tags(&note_id, tag_ids).map_err(|e| e.to_string())
}
