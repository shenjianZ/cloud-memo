use super::ErrorResponse;
use crate::middleware::logging::{log_info, RequestId};
use axum::{Extension, Json};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct CreateFolderRequest {
    pub name: String,
    pub parent_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct Folder {
    pub id: String,
    pub name: String,
    pub parent_id: Option<String>,
}

pub async fn list_folders(
    Extension(request_id): Extension<RequestId>,
) -> Result<Json<Vec<Folder>>, ErrorResponse> {
    log_info(&request_id, "列出文件夹请求", "");
    // TODO: 实现列出文件夹逻辑
    log_info(&request_id, "列出文件夹", "TODO: 未实现");
    Err(ErrorResponse::new("文件夹功能暂未实现"))
}

pub async fn create_folder(
    Extension(request_id): Extension<RequestId>,
    Json(req): Json<CreateFolderRequest>,
) -> Result<Json<Folder>, ErrorResponse> {
    log_info(&request_id, "创建文件夹请求", &req);
    // TODO: 实现创建文件夹逻辑
    log_info(&request_id, "创建文件夹", "TODO: 未实现");
    Err(ErrorResponse::new("文件夹功能暂未实现"))
}
