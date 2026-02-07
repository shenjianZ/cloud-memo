use crate::services::{SyncService, AppSettingsService};
use crate::models::error::Result;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{interval, Duration};

/// 自动同步服务
///
/// 提供定时自动同步功能，可配置同步间隔
#[derive(Clone)]
pub struct AutoSyncService {
    sync_service: SyncService,
    app_settings_service: AppSettingsService,
    is_running: Arc<Mutex<bool>>,
    manual_sync_in_progress: Arc<Mutex<bool>>,
    handle: Arc<Mutex<Option<tokio::task::JoinHandle<()>>>>,
}

impl AutoSyncService {
    /// 创建新的 AutoSyncService 实例
    pub fn new(
        sync_service: SyncService,
        app_settings_service: AppSettingsService,
    ) -> Self {
        Self {
            sync_service,
            app_settings_service,
            is_running: Arc::new(Mutex::new(false)),
            manual_sync_in_progress: Arc::new(Mutex::new(false)),
            handle: Arc::new(Mutex::new(None)),
        }
    }

    /// 启动自动同步服务
    pub async fn start(&self) -> Result<()> {
        log::info!("[AutoSyncService] 正在启动自动同步服务");

        let mut is_running = self.is_running.lock().await;
        if *is_running {
            log::info!("[AutoSyncService] 自动同步服务已在运行");
            return Ok(());
        }
        *is_running = true;
        drop(is_running);

        let sync_service = self.sync_service.clone();
        let app_settings_service = self.app_settings_service.clone();
        let is_running = self.is_running.clone();
        let manual_sync_in_progress = self.manual_sync_in_progress.clone();

        let task = tokio::spawn(async move {
            log::info!("[AutoSyncService] 后台同步任务已启动");
            let mut ticker = interval(Duration::from_secs(60)); // 每分钟检查一次
            ticker.tick().await; // 跳过第一次立即触发

            loop {
                ticker.tick().await;

                // 检查是否应该停止
                {
                    let running = is_running.lock().await;
                    if !*running {
                        log::info!("[AutoSyncService] 收到停止信号，退出后台任务");
                        break;
                    }
                }

                // 检查用户是否正在手动同步
                {
                    let manual_syncing = manual_sync_in_progress.lock().await;
                    if *manual_syncing {
                        log::info!("[AutoSyncService] 用户正在手动同步，跳过本次自动同步");
                        continue;
                    }
                }

                // 读取应用设置
                let settings = match app_settings_service.get_settings() {
                    Ok(s) => s,
                    Err(e) => {
                        log::error!("[AutoSyncService] 读取应用设置失败: {}", e);
                        continue;
                    }
                };

                // 检查是否启用自动同步
                if !settings.auto_sync_enabled {
                    log::debug!("[AutoSyncService] 自动同步未启用，跳过");
                    continue;
                }

                // 检查是否到同步时间
                let sync_interval_seconds = settings.sync_interval_minutes as i64 * 60;
                let now = chrono::Utc::now().timestamp();
                let last_sync_at = match sync_service.get_last_sync_at() {
                    Ok(Some(t)) => t,
                    Ok(None) => 0,
                    Err(e) => {
                        log::error!("[AutoSyncService] 获取上次同步时间失败: {}", e);
                        0  // 失败时使用 0，立即触发同步
                    }
                };

                if last_sync_at + sync_interval_seconds > now {
                    log::debug!(
                        "[AutoSyncService] 未到同步时间（上次同步: {}, 间隔: {}秒, 当前: {}）",
                        last_sync_at,
                        sync_interval_seconds,
                        now
                    );
                    continue;
                }

                // 执行自动同步
                log::info!("[AutoSyncService] 开始执行自动同步");
                match sync_service.full_sync().await {
                    Ok(report) => {
                        log::info!(
                            "[AutoSyncService] 自动同步成功: pushed_notes={}, pulled_notes={}, pulled_tags={}, conflicts={}",
                            report.pushed_notes,
                            report.pulled_notes,
                            report.pulled_tags,
                            report.conflict_count
                        );
                    }
                    Err(e) => {
                        log::error!("[AutoSyncService] 自动同步失败: {}", e);
                    }
                }
            }

            log::info!("[AutoSyncService] 后台同步任务已退出");
        });

        // 保存任务句柄
        let mut handle_guard = self.handle.lock().await;
        *handle_guard = Some(task);
        drop(handle_guard);

        log::info!("[AutoSyncService] 自动同步服务启动成功");
        Ok(())
    }

    /// 停止自动同步服务
    pub async fn stop(&self) {
        log::info!("[AutoSyncService] 正在停止自动同步服务");

        // 设置停止标志
        {
            let mut is_running = self.is_running.lock().await;
            *is_running = false;
        }

        // 注意：我们不立即等待任务结束，让它在下次循环检查时自然退出
        log::info!("[AutoSyncService] 自动同步服务停止信号已发送");
    }

    /// 标记用户手动同步开始（自动同步将跳过本次）
    pub async fn begin_manual_sync(&self) {
        log::info!("[AutoSyncService] 用户开始手动同步");
        let mut flag = self.manual_sync_in_progress.lock().await;
        *flag = true;
    }

    /// 标记用户手动同步结束
    pub async fn end_manual_sync(&self) {
        log::info!("[AutoSyncService] 用户手动同步结束");
        let mut flag = self.manual_sync_in_progress.lock().await;
        *flag = false;
    }

    /// 检查是否正在运行
    pub async fn is_running(&self) -> bool {
        *self.is_running.lock().await
    }

    /// 检查用户是否正在手动同步
    pub async fn is_manual_syncing(&self) -> bool {
        *self.manual_sync_in_progress.lock().await
    }
}
