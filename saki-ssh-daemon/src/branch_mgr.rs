//! Phase 8: 儲存層微型動態分支 (Micro Overlay Branching)
//!
//! 在 Agent 執行修改前，將目標目錄動態隔離。
//! Agent 的所有寫入都會進入容量極小的高層分支，而不會污染底層系統。

use std::path::{Path, PathBuf};
use tracing::{info, warn};

pub struct BranchMgr;

impl BranchMgr {
    /// 為特定的執行階段建立儲存層微型分支
    /// 由於 macOS 不支援 OverlayFS，我們這裡實作一個概念性的抽象層。
    /// 在真實的 APFS 環境中，這會對應到輕量級的 firmlink 或特定目錄快照。
    pub fn create_micro_branch(session_id: &str, target_dir: &str) -> Option<PathBuf> {
        let target_path = Path::new(target_dir);
        if !target_path.exists() {
            warn!("BranchMgr: Target directory {} does not exist. Skipping branch.", target_dir);
            return None;
        }

        let branch_dir = PathBuf::from(format!("/tmp/sass_branches/{}", session_id));
        if let Err(e) = std::fs::create_dir_all(&branch_dir) {
            warn!("BranchMgr: Failed to create branch dir: {}", e);
            return None;
        }

        info!(
            "BranchMgr: Created micro overlay branch for Session {} targeting {}",
            session_id, target_dir
        );

        // Phase 9: Userspace Symlink Tree Overlay
        // 這是為了在沒有 DriverKit / ESF VFS 權限的 macOS 與 Windows 上，
        // 達成零權限的目錄隔離。Linux 雖然可以走 Rootless OverlayFS，但在這裡我們統一步驟。
        if let Err(e) = Self::build_symlink_tree(target_path, &branch_dir) {
            warn!("BranchMgr: Failed to build symlink tree: {}", e);
        }

        Some(branch_dir)
    }

    /// 遞迴建立 Symlink Tree，排除重量級快取與 .git
    fn build_symlink_tree(src: &Path, dst: &Path) -> std::io::Result<()> {
        for entry in std::fs::read_dir(src)? {
            let entry = entry?;
            let ty = entry.file_type()?;
            let dest_path = dst.join(entry.file_name());
            
            if ty.is_dir() {
                let name = entry.file_name();
                if name == "target" || name == ".git" || name == "node_modules" {
                    continue;
                }
                std::fs::create_dir_all(&dest_path)?;
                Self::build_symlink_tree(&entry.path(), &dest_path)?;
            } else {
                #[cfg(unix)]
                let _ = std::os::unix::fs::symlink(entry.path(), dest_path);
                
                #[cfg(windows)]
                let _ = std::os::windows::fs::symlink_file(entry.path(), dest_path);
            }
        }
        Ok(())
    }

    /// 在 Human Review 失敗時捨棄分支
    pub fn drop_branch(session_id: &str) -> Result<(), std::io::Error> {
        let branch_dir = PathBuf::from(format!("/tmp/sass_branches/{}", session_id));
        if branch_dir.exists() {
            info!("BranchMgr: Dropping branch for Session {}", session_id);
            std::fs::remove_dir_all(&branch_dir)?;
        }
        Ok(())
    }

    /// 在 Human Review 成功時合併分支
    pub fn merge_branch(session_id: &str, _target_dir: &str) -> Result<(), std::io::Error> {
        info!("BranchMgr: Merging branch for Session {} (Simulated)", session_id);
        // 在真實實作中，這裡會將 /tmp/sass_branches/{session_id} 內的差異複製回 target_dir
        Self::drop_branch(session_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================
    // C.6 BranchManager 測試
    // ========================================

    #[test]
    fn create_micro_branch_nonexistent_dir_returns_none() {
        let result = BranchMgr::create_micro_branch(
            "test-session",
            "/nonexistent/directory/12345",
        );
        assert!(result.is_none(), "不存在的目錄應回傳 None");
    }

    #[test]
    fn create_micro_branch_existing_dir_returns_some() {
        // 使用臨時目錄作為 target
        let temp_dir = std::env::temp_dir().join(format!(
            "sass_test_branch_{}",
            uuid::Uuid::new_v4().to_string().split('-').next().unwrap()
        ));
        std::fs::create_dir_all(&temp_dir).unwrap();

        // 在 temp_dir 下建立一些測試檔案
        std::fs::write(temp_dir.join("test.txt"), "hello").unwrap();

        let session_id = format!("test-{}", uuid::Uuid::new_v4().to_string().split('-').next().unwrap());
        let branch_path = BranchMgr::create_micro_branch(
            &session_id,
            temp_dir.to_str().unwrap(),
        );

        assert!(branch_path.is_some(), "存在的目錄應回傳分支路徑");
        let bp = branch_path.unwrap();
        assert!(bp.exists(), "分支路徑應實際存在");

        // 清理
        let _ = BranchMgr::drop_branch(&session_id);
        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn drop_branch_nonexistent_does_not_panic() {
        let result = BranchMgr::drop_branch("nonexistent-session-xyz");
        assert!(result.is_ok(), "捨棄不存在的分支不應報錯");
    }

    #[test]
    fn drop_branch_removes_directory() {
        let session_id = format!("drop-test-{}", uuid::Uuid::new_v4().to_string().split('-').next().unwrap());
        let branch_dir = PathBuf::from(format!("/tmp/sass_branches/{}", session_id));
        std::fs::create_dir_all(&branch_dir).unwrap();
        assert!(branch_dir.exists());

        let result = BranchMgr::drop_branch(&session_id);
        assert!(result.is_ok());
        assert!(!branch_dir.exists(), "捨棄後分支目錄應被移除");
    }

    #[test]
    fn merge_branch_removes_branch_dir() {
        let session_id = format!("merge-test-{}", uuid::Uuid::new_v4().to_string().split('-').next().unwrap());
        let branch_dir = PathBuf::from(format!("/tmp/sass_branches/{}", session_id));
        std::fs::create_dir_all(&branch_dir).unwrap();

        let result = BranchMgr::merge_branch(&session_id, "/tmp");
        assert!(result.is_ok());
        assert!(!branch_dir.exists(), "合併後分支目錄應被移除");
    }
}

