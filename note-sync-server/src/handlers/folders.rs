use super::ErrorResponse;
use crate::middleware::logging::{log_info, RequestId};
use crate::models::Folder;
use crate::AppState;
use axum::{Extension, Json};
use axum::extract::State;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct CreateFolderRequest {
    pub name: String,
    pub parent_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ListFoldersRequest {
    pub workspace_id: Option<String>,
}

/// 列出文件夹
pub async fn list_folders(
    Extension(request_id): Extension<RequestId>,
    State(state): State<AppState>,
    Extension(user_id): Extension<String>,
    Json(req): Json<ListFoldersRequest>,
) -> Result<Json<Vec<Folder>>, ErrorResponse> {
    log_info(&request_id, "列出文件夹请求", &format!("user_id={}, workspace_id={:?}", user_id, req.workspace_id));

    let folders = if let Some(workspace_id) = req.workspace_id {
        // 指定工作空间
        sqlx::query_as::<_, Folder>(
            "SELECT * FROM folders
             WHERE user_id = ? AND workspace_id = ? AND is_deleted = FALSE
             ORDER BY created_at DESC"
        )
        .bind(&user_id)
        .bind(&workspace_id)
        .fetch_all(&state.pool)
        .await
    } else {
        // 查询默认空间
        sqlx::query_as::<_, Folder>(
            "SELECT f.* FROM folders f
             INNER JOIN workspaces w ON f.workspace_id = w.id
             WHERE f.user_id = ? AND w.user_id = ? AND w.is_default = TRUE
               AND f.is_deleted = FALSE AND w.is_deleted = FALSE
             ORDER BY f.created_at DESC"
        )
        .bind(&user_id)
        .bind(&user_id)
        .fetch_all(&state.pool)
        .await
    };

    let folders = folders.map_err(|e| {
        log_info(&request_id, "查询文件夹失败", &e.to_string());
        ErrorResponse::new("查询文件夹失败")
    })?;

    log_info(&request_id, "查询文件夹成功", &format!("count={}", folders.len()));
    Ok(Json(folders))
}

/// 创建文件夹
pub async fn create_folder(
    Extension(request_id): Extension<RequestId>,
    State(state): State<AppState>,
    Extension(user_id): Extension<String>,
    Json(req): Json<CreateFolderRequest>,
) -> Result<Json<Folder>, ErrorResponse> {
    log_info(&request_id, "创建文件夹请求", &format!("user_id={}, name={}", user_id, req.name));

    let folder_id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().timestamp();

    // TODO: 如果未指定 workspace_id，需要使用用户的默认空间
    let workspace_id: Option<String> = None; // 暂时使用 None，待实现查询默认空间

    let _result = sqlx::query(
        "INSERT INTO folders (id, user_id, workspace_id, name, parent_id, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(&folder_id)
    .bind(&user_id)
    .bind(&workspace_id)
    .bind(&req.name)
    .bind(&req.parent_id)
    .bind(now)
    .bind(now)
    .execute(&state.pool)
    .await
    .map_err(|e| {
        log_info(&request_id, "创建文件夹失败", &format!("error={}", e));
        ErrorResponse::new("创建文件夹失败")
    })?;

    let folder = sqlx::query_as::<_, Folder>("SELECT * FROM folders WHERE id = ?")
        .bind(&folder_id)
        .fetch_one(&state.pool)
        .await
        .map_err(|e| {
            log_info(&request_id, "查询新创建的文件夹失败", &format!("error={}", e));
            ErrorResponse::new("查询文件夹失败")
        })?;

    log_info(&request_id, "创建文件夹成功", &format!("folder_id={}", folder_id));
    Ok(Json(folder))
}
