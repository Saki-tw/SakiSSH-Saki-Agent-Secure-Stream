#![no_std]
#![no_main]

use aya_bpf::{
    macros::{lsm, map},
    maps::HashMap,
    programs::LsmContext,
};

// 儲存被 SASS 閹割的 PID 列表
#[map]
static RESTRICTED_PIDS: HashMap<u32, u32> = HashMap::with_max_entries(1024, 0);

/// LSM 掛載點：防禦透過 symlink 產生的 NTFS Junction / 空間扭曲
#[lsm(hook = "path_symlink")]
pub fn saki_agent_path_symlink(ctx: LsmContext) -> i32 {
    let pid = (aya_bpf::helpers::bpf_get_current_pid_tgid() >> 32) as u32;

    // 如果此 PID 被登記在受限名單中，物理阻斷 (回傳 -EPERM = -1)
    if unsafe { RESTRICTED_PIDS.get(&pid).is_some() } {
        return -1; 
    }

    // 允許其他人正常運作
    0
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    unsafe { core::hint::unreachable_unchecked() }
}
