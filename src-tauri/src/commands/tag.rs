use crate::services::TagService;
use crate::models::{Tag, CreateTagRequest, UpdateTagRequest, NoteTagRequest};
use tauri::State;

type TagSvc<'a> = State<'a, TagService>;

/// 获取所有标签
#[tauri::command]
pub async fn get_all_tags(
    service: TagSvc<'_>,
) -> std::result::Result<Vec<Tag>, String> {
    log::debug!("[commands/tag.rs::get_all_tags] 获取所有标签");

    service.get_all_tags()
        .map_err(|e| {
            log::error!("[commands/tag.rs::get_all_tags] 获取失败: {}", e);
            e.to_string()
        })
        .map(|tags| {
            log::debug!("[commands/tag.rs::get_all_tags] 获取成功: count={}", tags.len());
            tags
        })
}

/// 根据 ID 获取标签
#[tauri::command]
pub async fn get_tag(
    id: String,
    service: TagSvc<'_>,
) -> std::result::Result<Tag, String> {
    log::debug!("[commands/tag.rs::get_tag] 获取标签: id={}", id);

    service.get_tag(&id)
        .map_err(|e| {
            log::error!("[commands/tag.rs::get_tag] 获取失败: id={}, error={}", id, e);
            e.to_string()
        })
}

/// 获取笔记的所有标签
#[tauri::command]
pub async fn get_note_tags(
    note_id: String,
    service: TagSvc<'_>,
) -> std::result::Result<Vec<Tag>, String> {
    log::debug!("[commands/tag.rs::get_note_tags] 获取笔记的标签: note_id={}", note_id);

    service.get_note_tags(&note_id)
        .map_err(|e| {
            log::error!("[commands/tag.rs::get_note_tags] 获取失败: note_id={}, error={}", note_id, e);
            e.to_string()
        })
        .map(|tags| {
            log::debug!("[commands/tag.rs::get_note_tags] 获取成功: note_id={}, count={}", note_id, tags.len());
            tags
        })
}

/// 创建标签
#[tauri::command]
pub async fn create_tag(
    req: CreateTagRequest,
    service: TagSvc<'_>,
) -> std::result::Result<Tag, String> {
    let name = req.name.clone();
    log::info!("[commands/tag.rs::create_tag] 创建标签: name={}", name);

    service.create_tag(req)
        .map_err(|e| {
            log::error!("[commands/tag.rs::create_tag] 创建失败: {}", e);
            e.to_string()
        })
        .map(|tag| {
            log::info!("[commands/tag.rs::create_tag] 创建成功: id={}, name={}", tag.id, tag.name);
            tag
        })
}

/// 更新标签
#[tauri::command]
pub async fn update_tag(
    id: String,
    req: UpdateTagRequest,
    service: TagSvc<'_>,
) -> std::result::Result<Tag, String> {
    let name_display = req.name.as_deref().unwrap_or("(未修改)");
    log::debug!("[commands/tag.rs::update_tag] 更新标签: id={}, name={}", id, name_display);

    service.update_tag(&id, req)
        .map_err(|e| {
            log::error!("[commands/tag.rs::update_tag] 更新失败: id={}, error={}", id, e);
            e.to_string()
        })
        .map(|tag| {
            log::debug!("[commands/tag.rs::update_tag] 更新成功: id={}", id);
            tag
        })
}

/// 删除标签
#[tauri::command]
pub async fn delete_tag(
    id: String,
    service: TagSvc<'_>,
) -> std::result::Result<(), String> {
    log::info!("[commands/tag.rs::delete_tag] 删除标签: id={}", id);

    service.delete_tag(&id)
        .map_err(|e| {
            log::error!("[commands/tag.rs::delete_tag] 删除失败: id={}, error={}", id, e);
            e.to_string()
        })
        .map(|_| {
            log::info!("[commands/tag.rs::delete_tag] 删除成功: id={}", id);
        })
}

/// 为笔记添加标签
#[tauri::command]
pub async fn add_tag_to_note(
    req: NoteTagRequest,
    service: TagSvc<'_>,
) -> std::result::Result<(), String> {
    log::info!("[commands/tag.rs::add_tag_to_note] 为笔记添加标签: note_id={}, tag_id={}", req.note_id, req.tag_id);

    service.add_tag_to_note(req)
        .map_err(|e| {
            log::error!("[commands/tag.rs::add_tag_to_note] 添加失败: {}", e);
            e.to_string()
        })
        .map(|_| {
            log::info!("[commands/tag.rs::add_tag_to_note] 添加成功");
        })
}

/// 从笔记移除标签
#[tauri::command]
pub async fn remove_tag_from_note(
    note_id: String,
    tag_id: String,
    service: TagSvc<'_>,
) -> std::result::Result<(), String> {
    log::info!("[commands/tag.rs::remove_tag_from_note] 从笔记移除标签: note_id={}, tag_id={}", note_id, tag_id);

    service.remove_tag_from_note(&note_id, &tag_id)
        .map_err(|e| {
            log::error!("[commands/tag.rs::remove_tag_from_note] 移除失败: {}", e);
            e.to_string()
        })
        .map(|_| {
            log::info!("[commands/tag.rs::remove_tag_from_note] 移除成功");
        })
}

/// 设置笔记的标签（替换所有标签）
#[tauri::command]
pub async fn set_note_tags(
    note_id: String,
    tag_ids: Vec<String>,
    service: TagSvc<'_>,
) -> std::result::Result<(), String> {
    log::info!("[commands/tag.rs::set_note_tags] 设置笔记标签: note_id={}, count={}", note_id, tag_ids.len());

    service.set_note_tags(&note_id, tag_ids)
        .map_err(|e| {
            log::error!("[commands/tag.rs::set_note_tags] 设置失败: note_id={}, error={}", note_id, e);
            e.to_string()
        })
        .map(|_| {
            log::info!("[commands/tag.rs::set_note_tags] 设置成功: note_id={}", note_id);
        })
}
