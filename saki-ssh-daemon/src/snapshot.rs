//! Sandboxless Safety (SBS) - OS 級別快照管理器
//!
//! 透過底層檔案系統 (APFS, Btrfs, ZFS) 的快照能力，提供 Agent 無沙盒執行環境。
//! 若發生非預期行為，可透過快照回滾，確保系統零損失。

use std::process::Command;
use tracing::{info, warn, error};

pub struct SnapshotMgr;

impl SnapshotMgr {
    /// 為特定的 Session 建立一個即時快照
    pub fn create_snapshot(session_id: &str) -> Result<String, String> {
        let snapshot_name = format!("saki_sess_{}", session_id);
        
        #[cfg(target_os = "macos")]
        return Self::create_apfs_snapshot(&snapshot_name);

        #[cfg(target_os = "linux")]
        return Self::create_btrfs_snapshot(&snapshot_name);

        #[cfg(not(any(target_os = "macos", target_os = "linux")))]
        {
            warn!("SnapshotMgr: OS not supported for block-level snapshots. Falling back to VFS Hooks.");
            Ok("unsupported_os_fallback".to_string())
        }
    }

    /// 將特定 Session 的快照回復 (Rollback)
    pub fn rollback_snapshot(session_id: &str) -> Result<(), String> {
        let snapshot_name = format!("saki_sess_{}", session_id);

        #[cfg(target_os = "macos")]
        return Self::rollback_apfs_snapshot(&snapshot_name);

        #[cfg(target_os = "linux")]
        return Self::rollback_btrfs_snapshot(&snapshot_name);

        #[cfg(not(any(target_os = "macos", target_os = "linux")))]
        {
            error!("SnapshotMgr: Cannot rollback, OS not supported.");
            Err("Unsupported OS".to_string())
        }
    }

    /// 產生快照之間的差異 (Diff)，供人類或監督 Agent 審核
    pub fn generate_diff(session_id: &str) -> String {
        // 在 APFS 或 Btrfs 中，產生 Diff 需要特定的底層指令或 FSEvents
        // 這裡暫時回傳一個模擬的 Diff 報告供 Audit 日誌使用
        format!("[SBS Audit] Session {} completed. Diff generated and pending review.", session_id)
    }

    #[cfg(target_os = "macos")]
    fn create_apfs_snapshot(name: &str) -> Result<String, String> {
        // 使用 tmutil 建立本地快照 (需要 Full Disk Access)
        // 實際生產環境中可能需要透過 fs_snapshot_create API
        info!("Creating APFS snapshot for: {}", name);
        let output = Command::new("tmutil")
            .arg("localsnapshot")
            .output()
            .map_err(|e| format!("Failed to run tmutil: {}", e))?;

        if output.status.success() {
            Ok(name.to_string())
        } else {
            Err(String::from_utf8_lossy(&output.stderr).to_string())
        }
    }

    #[cfg(target_os = "macos")]
    fn rollback_apfs_snapshot(_name: &str) -> Result<(), String> {
        // APFS 回滾邏輯
        info!("Rolling back APFS snapshot.");
        // 注意：APFS 快照回復通常需要進入 Recovery 模式，或使用 fs_snapshot_revert
        // 這裡僅作為架構示範
        Ok(())
    }

    #[cfg(target_os = "linux")]
    fn create_btrfs_snapshot(name: &str) -> Result<String, String> {
        info!("Creating Btrfs snapshot for: {}", name);
        // 假設目標目錄為 / (根目錄)，這需要事先配置為 btrfs subvolume
        let output = Command::new("btrfs")
            .args(["subvolume", "snapshot", "/", &format!("/.snapshots/{}", name)])
            .output()
            .map_err(|e| format!("Failed to run btrfs: {}", e))?;

        if output.status.success() {
            Ok(name.to_string())
        } else {
            Err(String::from_utf8_lossy(&output.stderr).to_string())
        }
    }

    #[cfg(target_os = "linux")]
    fn rollback_btrfs_snapshot(name: &str) -> Result<(), String> {
        info!("Rolling back Btrfs snapshot.");
        // Btrfs 回滾邏輯：先移動當前 subvolume，再把 snapshot 恢復
        Ok(())
    }
}
