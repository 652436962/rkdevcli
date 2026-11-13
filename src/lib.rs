//! Rockchip flashing engine shared by the `rkdevcli` command line tool and the
//! `rkdevsuit` desktop application.
//!
//! The crate has no GUI dependency by default. The optional `desktop` feature
//! only adds host adapters (file pickers, MCP approval dialogs and window
//! events) that a GUI shell can wire up; USB, image and job handling stay
//! toolkit independent.

pub mod advanced;
pub mod archive;
#[cfg(feature = "headless-cli")]
pub mod cli;
#[cfg(feature = "desktop")]
pub mod dialogs;
pub mod firmware;
pub mod gpt;
pub mod host;
pub mod jobs;
pub mod loader;
pub mod local_jobs;
#[cfg(feature = "maintenance-cli")]
pub mod maintenance;
pub mod mcp;
pub mod model;
pub mod state;
pub mod usb;

use model::*;
use state::Backend;
use std::time::Duration;
use tokio::sync::OwnedMutexGuard;

/// Enumerate connected RockUSB targets, excluding devices that are currently
/// rebooting, and enrich Loader entries with chip and capacity information.
pub async fn scan(backend: &Backend) -> Result<Vec<DeviceInfo>, String> {
    let _gate = backend
        .usb_gate
        .clone()
        .try_lock_owned()
        .map_err(|_| "设备任务运行中，扫描已暂停")?;
    let mut targets = usb::targets().await?;
    let connected_ids: Vec<_> = targets
        .iter()
        .map(|target| target.info.id.as_str())
        .collect();
    let rebooting_ids = backend.rebooting_device_ids(&connected_ids).await;
    targets.retain(|target| !rebooting_ids.contains(&target.info.id));
    probe_targets(targets).await
}

pub async fn probe_targets(targets: Vec<usb::Target>) -> Result<Vec<DeviceInfo>, String> {
    let mut devices = Vec::new();
    for mut target in targets {
        usb::detect_handoff(&mut target).await;
        if target.info.mode == "loader" {
            // These values enrich the device list; unsupported metadata
            // commands should not hide the device or hold the scan gate for
            // a regular transfer timeout. Drop the session on a failed read.
            let _ = tokio::time::timeout(Duration::from_secs(1), async {
                let mut session = usb::Session::open(&target).await?;
                target.info.capacity = Some(session.capacity().await? * 512);
                session.describe_chip(&mut target).await
            })
            .await;
        }
        devices.push(target.info);
    }
    Ok(devices)
}

pub async fn inspect_firmware(path: String) -> Result<FirmwareInfo, String> {
    tokio::task::spawn_blocking(move || firmware::inspect(&path).map(|image| image.info))
        .await
        .map_err(|e| format!("固件检查任务异常：{e}"))?
}

pub async fn inspect_archive(path: String) -> Result<archive::ArchiveInfo, String> {
    tokio::task::spawn_blocking(move || archive::inspect_archive(&path))
        .await
        .map_err(|e| format!("压缩包检查任务异常：{e}"))?
}

pub async fn partitions(backend: &Backend, device_id: &str) -> Result<Vec<PartitionInfo>, String> {
    let _gate = backend
        .usb_gate
        .clone()
        .try_lock_owned()
        .map_err(|_| "设备任务运行中，暂不能读取分区")?;
    let mut target = usb::resolve(device_id).await?;
    usb::detect_handoff(&mut target).await;
    if target.info.mode != "loader" {
        return Err("读取分区需要 Loader 模式；请先下载 Loader".into());
    }
    let mut session = usb::Session::open(&target).await?;
    usb::partitions(&mut session).await
}

/// Validate a write request and reserve exclusive USB access. The returned
/// guard must be passed to [`jobs::launch`] so the reservation is held for the
/// whole operation.
pub async fn prepare_operation(
    backend: &Backend,
    request: OperationRequest,
) -> Result<(jobs::Prepared, OwnedMutexGuard<()>), String> {
    let gate = backend
        .usb_gate
        .clone()
        .try_lock_owned()
        .map_err(|_| "USB 设备正忙；请等待当前任务或扫描完成")?;
    if backend.is_busy() {
        return Err("已有任务正在运行，请勿重复提交".into());
    }
    Ok((jobs::prepare(request).await?, gate))
}

pub fn runtime_info() -> RuntimeInfo {
    RuntimeInfo {
        version: env!("CARGO_PKG_VERSION").into(),
        platform: format!("{} / {}", std::env::consts::OS, std::env::consts::ARCH),
        backend: "RockUSB 0.4 · nusb 原生 USB（无需命令行工具）".into(),
    }
}
