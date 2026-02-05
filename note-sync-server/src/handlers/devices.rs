use axum::{Json, extract::{Path, State}, Extension};
use serde::Deserialize;
use axum::http::StatusCode;
use crate::AppState;
use crate::services::device_service::DeviceService;
use crate::models::Device;
use crate::middleware::logging::{RequestId, log_info};
use super::ErrorResponse;

#[derive(Debug, Deserialize)]
pub struct RegisterDeviceRequest {
    pub device_name: String,
    pub device_type: Option<String>,  // 可选，如果不提供会自动检测
}

/// 获取用户的所有设备
pub async fn list_devices(
    Extension(request_id): Extension<RequestId>,
    State(state): State<AppState>,
    Extension(user_id): Extension<String>,
) -> Result<Json<Vec<Device>>, ErrorResponse> {
    log_info(&request_id, "获取设备列表", &format!("user_id={}", user_id));

    let service = DeviceService::new(state.pool);

    match service.list_devices(&user_id).await {
        Ok(devices) => {
            log_info(&request_id, "获取成功", &format!("设备数量={}", devices.len()));
            Ok(Json(devices))
        }
        Err(e) => {
            log_info(&request_id, "获取失败", &e.to_string());
            Err(ErrorResponse::new("获取设备列表失败"))
        }
    }
}

/// 注册新设备（登录时自动调用）
pub async fn register_device(
    Extension(request_id): Extension<RequestId>,
    State(state): State<AppState>,
    Extension(user_id): Extension<String>,
    Json(req): Json<RegisterDeviceRequest>,
) -> Result<Json<Device>, ErrorResponse> {
    log_info(&request_id, "注册设备请求", &format!("user_id={}, device_name={}", user_id, req.device_name));

    let service = DeviceService::new(state.pool);

    // 如果没有提供 device_type，根据 User-Agent 检测
    let device_type = req.device_type.unwrap_or_else(|| {
        DeviceService::parse_device_type(None).to_string()
    });

    match service
        .register_or_update(&user_id, &req.device_name, &device_type)
        .await
    {
        Ok(device) => {
            log_info(&request_id, "注册成功", &format!("device_id={}", device.id));
            Ok(Json(device))
        }
        Err(e) => {
            log_info(&request_id, "注册失败", &e.to_string());
            Err(ErrorResponse::new("注册设备失败"))
        }
    }
}

/// 撤销设备
pub async fn revoke_device(
    Extension(request_id): Extension<RequestId>,
    State(state): State<AppState>,
    Extension(user_id): Extension<String>,
    Path(device_id): Path<String>,
) -> Result<StatusCode, ErrorResponse> {
    log_info(&request_id, "撤销设备请求", &format!("user_id={}, device_id={}", user_id, device_id));

    let service = DeviceService::new(state.pool);

    match service.revoke_device(&device_id, &user_id).await {
        Ok(_) => {
            log_info(&request_id, "撤销成功", "");
            Ok(StatusCode::OK)
        }
        Err(e) => {
            log_info(&request_id, "撤销失败", &e.to_string());
            if e.to_string().contains("not found") {
                Err(ErrorResponse::new("设备不存在"))
            } else {
                Err(ErrorResponse::new("撤销设备失败"))
            }
        }
    }
}

/// 设备心跳
pub async fn device_heartbeat(
    Extension(request_id): Extension<RequestId>,
    State(state): State<AppState>,
    Extension(user_id): Extension<String>,
    Path(device_id): Path<String>,
) -> Result<StatusCode, ErrorResponse> {
    log_info(&request_id, "设备心跳请求", &format!("user_id={}, device_id={}", user_id, device_id));

    let service = DeviceService::new(state.pool);

    match service.update_heartbeat(&device_id, &user_id).await {
        Ok(_) => {
            log_info(&request_id, "心跳更新成功", "");
            Ok(StatusCode::OK)
        }
        Err(e) => {
            log_info(&request_id, "心跳更新失败", &e.to_string());
            if e.to_string().contains("not found") {
                Err(ErrorResponse::new("设备不存在"))
            } else {
                Err(ErrorResponse::new("更新心跳失败"))
            }
        }
    }
}
