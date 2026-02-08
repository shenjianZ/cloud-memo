use super::ErrorResponse;
use crate::middleware::logging::{log_info, RequestId};
use crate::models::{Workspace, CreateWorkspaceRequest, UpdateWorkspaceRequest};
use crate::AppState;
use axum::{Extension, Json};
use axum::extract::State;

pub async fn list_workspaces(
    Extension(request_id): Extension<RequestId>,
    State(state): State<AppState>,
    Extension(user_id): Extension<String>,
) -> Result<Json<Vec<Workspace>>, ErrorResponse> {
    log_info(&request_id, "列出工作空间请求", &format!("user_id={}", user_id));

    let workspaces = sqlx::query_as::<_, Workspace>(
        "SELECT * FROM workspaces WHERE user_id = ? AND is_deleted = FALSE ORDER BY sort_order, created_at"
    )
    .bind(&user_id)
    .fetch_all(&state.pool)
    .await
    .map_err(|e| {
        log_info(&request_id, "查询工作空间失败", &format!("error={}", e));
        ErrorResponse::new("查询工作空间失败")
    })?;

    log_info(&request_id, "查询工作空间成功", &format!("count={}", workspaces.len()));
    Ok(Json(workspaces))
}

pub async fn create_workspace(
    Extension(request_id): Extension<RequestId>,
    State(state): State<AppState>,
    Extension(user_id): Extension<String>,
    Json(req): Json<CreateWorkspaceRequest>,
) -> Result<Json<Workspace>, ErrorResponse> {
    log_info(&request_id, "创建工作空间请求", &format!("user_id={}, name={}", user_id, req.name));

    let workspace_id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().timestamp();

    let _workspace = sqlx::query(
        "INSERT INTO workspaces (id, user_id, name, description, icon, color, is_default, sort_order, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?, FALSE, ?, ?, ?)"
    )
    .bind(&workspace_id)
    .bind(&user_id)
    .bind(&req.name)
    .bind(&req.description)
    .bind(&req.icon)
    .bind(&req.color)
    .bind(0) // sort_order
    .bind(now)
    .bind(now)
    .execute(&state.pool)
    .await
    .map_err(|e| {
        log_info(&request_id, "创建工作空间失败", &format!("error={}", e));
        ErrorResponse::new("创建工作空间失败")
    })?;

    let workspace = sqlx::query_as::<_, Workspace>("SELECT * FROM workspaces WHERE id = ?")
        .bind(&workspace_id)
        .fetch_one(&state.pool)
        .await
        .map_err(|e| {
            log_info(&request_id, "查询新创建的工作空间失败", &format!("error={}", e));
            ErrorResponse::new("查询工作空间失败")
        })?;

    log_info(&request_id, "创建工作空间成功", &format!("workspace_id={}", workspace_id));
    Ok(Json(workspace))
}

pub async fn update_workspace(
    Extension(request_id): Extension<RequestId>,
    State(state): State<AppState>,
    Extension(user_id): Extension<String>,
    axum::extract::Path(id): axum::extract::Path<String>,
    Json(req): Json<UpdateWorkspaceRequest>,
) -> Result<Json<Workspace>, ErrorResponse> {
    log_info(&request_id, "更新工作空间请求", &format!("user_id={}, workspace_id={}", user_id, id));

    let now = chrono::Utc::now().timestamp();

    sqlx::query(
        "UPDATE workspaces
         SET name = COALESCE(?, name),
             description = COALESCE(?, description),
             icon = COALESCE(?, icon),
             color = COALESCE(?, color),
             updated_at = ?
         WHERE id = ? AND user_id = ? AND is_deleted = FALSE"
    )
    .bind(&req.name)
    .bind(&req.description)
    .bind(&req.icon)
    .bind(&req.color)
    .bind(now)
    .bind(&id)
    .bind(&user_id)
    .execute(&state.pool)
    .await
    .map_err(|e| {
        log_info(&request_id, "更新工作空间失败", &format!("error={}", e));
        ErrorResponse::new("更新工作空间失败")
    })?;

    let workspace = sqlx::query_as::<_, Workspace>("SELECT * FROM workspaces WHERE id = ?")
        .bind(&id)
        .fetch_one(&state.pool)
        .await
        .map_err(|e| {
            log_info(&request_id, "查询更新后的工作空间失败", &format!("error={}", e));
            ErrorResponse::new("查询工作空间失败")
        })?;

    log_info(&request_id, "更新工作空间成功", &format!("workspace_id={}", id));
    Ok(Json(workspace))
}

pub async fn delete_workspace(
    Extension(request_id): Extension<RequestId>,
    State(state): State<AppState>,
    Extension(user_id): Extension<String>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Result<Json<()>, ErrorResponse> {
    log_info(&request_id, "删除工作空间请求", &format!("user_id={}, workspace_id={}", user_id, id));

    // 检查是否为默认空间，不允许删除
    let is_default: bool = sqlx::query_scalar("SELECT is_default FROM workspaces WHERE id = ? AND user_id = ? AND is_deleted = FALSE")
        .bind(&id)
        .bind(&user_id)
        .fetch_optional(&state.pool)
        .await
        .map_err(|e| {
            log_info(&request_id, "查询工作空间失败", &format!("error={}", e));
            ErrorResponse::new("查询工作空间失败")
        })?
        .unwrap_or(false);

    if is_default {
        log_info(&request_id, "不允许删除默认空间", &format!("workspace_id={}", id));
        return Err(ErrorResponse::new("不允许删除默认空间"));
    }

    let now = chrono::Utc::now().timestamp();

    // 软删除工作空间
    sqlx::query("UPDATE workspaces SET is_deleted = TRUE, deleted_at = ? WHERE id = ? AND user_id = ?")
        .bind(now)
        .bind(&id)
        .bind(&user_id)
        .execute(&state.pool)
        .await
        .map_err(|e| {
            log_info(&request_id, "删除工作空间失败", &format!("error={}", e));
            ErrorResponse::new("删除工作空间失败")
        })?;

    log_info(&request_id, "删除工作空间成功", &format!("workspace_id={}", id));
    Ok(Json(()))
}

pub async fn set_default_workspace(
    Extension(request_id): Extension<RequestId>,
    State(state): State<AppState>,
    Extension(user_id): Extension<String>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Result<Json<()>, ErrorResponse> {
    log_info(&request_id, "设置默认工作空间请求", &format!("user_id={}, workspace_id={}", user_id, id));

    let mut tx = state.pool.begin().await.map_err(|e| {
        log_info(&request_id, "开始事务失败", &format!("error={}", e));
        ErrorResponse::new("开始事务失败")
    })?;

    // 取消所有默认空间
    sqlx::query("UPDATE workspaces SET is_default = FALSE WHERE user_id = ?")
        .bind(&user_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| {
            log_info(&request_id, "取消默认空间失败", &format!("error={}", e));
            ErrorResponse::new("取消默认空间失败")
        })?;

    // 设置新的默认空间
    sqlx::query("UPDATE workspaces SET is_default = TRUE WHERE id = ? AND user_id = ? AND is_deleted = FALSE")
        .bind(&id)
        .bind(&user_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| {
            log_info(&request_id, "设置默认空间失败", &format!("error={}", e));
            ErrorResponse::new("设置默认空间失败")
        })?;

    tx.commit().await.map_err(|e| {
        log_info(&request_id, "提交事务失败", &format!("error={}", e));
        ErrorResponse::new("提交事务失败")
    })?;

    log_info(&request_id, "设置默认工作空间成功", &format!("workspace_id={}", id));
    Ok(Json(()))
}
