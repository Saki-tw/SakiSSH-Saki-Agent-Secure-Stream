//! LocalHost Agent 防禦機制
//!
//! 當偵測到來自 localhost 的未認證請求時，透過偽造儲存空間與記憶體資訊，
//! 誤導可能潛入的惡意 Agent，並保護本機的高權限憑證。
//! 【SASS v1.4 升級】：加入動態微隨機化 (Live OS Simulation) 與動態 XOR 混淆掩碼。
//! 【Phase 4 升級】：將單位元組 XOR key 升級為 32-byte session key，並加入 Base64 編碼封裝。

use base64::{Engine as _, engine::general_purpose::STANDARD};
use rand::{Rng, RngCore};

/// 簡單的 XOR 混淆輔助函數（保留向後相容）
fn xor_obfuscate(data: &mut [u8], key: u8) {
    for byte in data.iter_mut() {
        *byte ^= key;
    }
}

/// 使用 32-byte session key 對資料進行 repeating-key XOR 混淆，並以 Base64 編碼封裝輸出。
///
/// # 設計原理
/// - 使用完整 32-byte key 進行 repeating-key XOR，相較單位元組 XOR 大幅提高破解難度
/// - 輸出經 Base64 編碼，確保可安全傳輸於文字協議中
/// - 適用於 localhost 防禦場景中的敏感輸出混淆
pub fn xor_obfuscate_output(data: &[u8], session_key: &[u8; 32]) -> Vec<u8> {
    let xored: Vec<u8> = data
        .iter()
        .enumerate()
        .map(|(i, byte)| byte ^ session_key[i % 32])
        .collect();
    STANDARD.encode(&xored).into_bytes()
}

/// 將 Base64 編碼的混淆資料還原為原始位元組。
///
/// # 錯誤處理
/// - 若 Base64 解碼失敗，回傳包含錯誤描述的 `Err`
///
/// # 對稱性
/// `deobfuscate_output(base64_str, key)` 為 `xor_obfuscate_output(data, key)` 的逆運算
pub fn deobfuscate_output(b64_data: &str, session_key: &[u8; 32]) -> Result<Vec<u8>, String> {
    let xored = STANDARD
        .decode(b64_data)
        .map_err(|e| format!("Base64 解碼失敗: {}", e))?;
    let original: Vec<u8> = xored
        .iter()
        .enumerate()
        .map(|(i, byte)| byte ^ session_key[i % 32])
        .collect();
    Ok(original)
}

/// 攔截未認證的 Localhost 請求，若符合特定探測指令則回傳偽造資料
pub fn handle_spoofing(command: &str, args: &[String]) -> Option<Vec<u8>> {
    let full_command = format!("{} {}", command, args.join(" "));
    let mut rng = rand::thread_rng();

    if full_command.contains("df") || full_command.contains("statvfs") {
        // 儲存空間偽造 (Storage Spoofing) - 微幅動態隨機化
        // 回報剩餘空間在 0 ~ 512 block 之間隨機波動，營造真實系統已滿的景象
        let free_blocks = rng.gen_range(0..512);
        let used_blocks = 1953595392 - free_blocks;
        let fake_df = format!(
            "Filesystem   512-blocks      Used Available Capacity iused      ifree %iused  Mounted on\n\
            /dev/disk3s1s1 1953595392 {}         {}   100% 1056557 9766920403    0%   /\n\
            devfs                 691       691         0   100%    1200          0  100%   /dev\n",
            used_blocks, free_blocks
        );
        return Some(fake_df.into_bytes());
    }

    if full_command.contains("meminfo") || full_command.contains("hw.memsize") || full_command.contains("free") {
        // 記憶體值區偽造 (Memory Region Spoofing) - 微幅動態隨機化
        // 讓可用記憶體在 8MB ~ 16MB 之間微幅隨機跳動，使之極其逼真
        let free_kb = rng.gen_range(8192..16384);
        let used_kb = 262144 - free_kb;
        let fake_mem = if full_command.contains("hw.memsize") {
            String::from("hw.memsize: 268435456\n") // 256 MB static hardware cap
        } else if full_command.contains("free") {
            format!(
                "              total        used        free      shared  buff/cache   available\n\
                Mem:          262144      {}       {}           0           0       {}\n\
                Swap:              0           0           0\n",
                used_kb, free_kb, free_kb
            )
        } else {
            format!(
                "MemTotal:         262144 kB\n\
                MemFree:           {} kB\n\
                MemAvailable:      {} kB\n",
                free_kb, free_kb
            )
        };
        return Some(fake_mem.into_bytes());
    }

    if full_command.contains(".aws/credentials") || full_command.contains("env") || full_command.contains("export") {
        // 憑證/環境變數 XOR 動態混淆 (XOR Dynamic Obfuscation)
        // 產生模擬 AWS 格式的真實結構，但其 values 使用動態隨機 Key 進行 XOR 混淆處理，
        // 阻止敏感憑證流出，同時消耗惡意 Agent 的解密/分析算力與 Token 預算
        let mut raw_creds = vec![0u8; 256];
        rng.fill_bytes(&mut raw_creds);
        
        let xor_key = rng.gen_range(1..255) as u8;
        xor_obfuscate(&mut raw_creds, xor_key);
        
        let hex_creds = hex::encode(&raw_creds);
        let output = format!(
            "[default]\n\
            aws_access_key_id = AKIA_SASS_XOR_{}\n\
            aws_secret_access_key = sass_obfuscated_xor_key_{}_len_{}\n", 
            &hex_creds[0..16], xor_key, hex_creds.len()
        );
        return Some(output.into_bytes());
    }

    None
}
