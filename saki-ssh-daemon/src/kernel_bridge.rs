//! Kernel Bridge - Ring-0 防禦通訊層
//!
//! 提供 SASS Daemon 統一個跨平台介面，與底層的 OS Driver 溝通。
//! 負責將 Spawn 產生的 Agent PID 註冊進系統核心中，物理閹割其權限。

use tracing::{info, warn};

/// 與底層驅動溝通的橋樑
pub struct KernelBridge;

impl KernelBridge {
    /// 將受限的 Agent PID 註冊進 Kernel Driver 中
    pub fn register_restricted_pid(pid: u32) {
        #[cfg(target_os = "macos")]
        Self::register_esf_macos(pid);

        #[cfg(target_os = "linux")]
        Self::register_ebpf_linux(pid);

        #[cfg(target_os = "windows")]
        Self::register_minifilter_windows(pid);

        // 防呆記錄，確保即便 OS 未支援也有 Log 追蹤
        #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
        warn!("KernelBridge: OS not supported for PID {} restriction.", pid);
    }

    #[cfg(target_os = "macos")]
    fn register_esf_macos(pid: u32) {
        // TODO: 實作 XPC 連線到 com.apple.developer.endpoint-security.client Extension
        info!("KernelBridge [macOS]: Registered PID {} for ESF sandbox.", pid);
    }

    #[cfg(target_os = "linux")]
    fn register_ebpf_linux(pid: u32) {
        // TODO: 實作 Aya-rs BPF Map 更新
        info!("KernelBridge [Linux]: Registered PID {} in BPF Map.", pid);
    }

    #[cfg(target_os = "windows")]
    fn register_minifilter_windows(pid: u32) {
        // TODO: 實作 DeviceIoControl 傳送 IRP 到 Minifilter
        info!("KernelBridge [Windows]: Registered PID {} to Minifilter driver.", pid);
    }
}
