use chacha20poly1305::{
    aead::{Aead, KeyInit},
    ChaCha20Poly1305, Nonce,
};
use rand::{rngs::OsRng, RngCore};
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::io::AsyncWriteExt;
use tracing::{warn, error, info};
use hmac::{Hmac, Mac};
use sha2::Sha256;

/// HMAC-SHA256 別名
type HmacSha256 = Hmac<Sha256>;

// ============================================================
// TLS Exporter 綁定 (RFC 5705 / RFC 8446 §7.5 Plugins 版本)
// ============================================================

/// TLS Exporter Label — 用於通道綁定的 Exported Keying Material (EKM)
/// 
/// 格式遵循 RFC 5705 §4: label 不得為 IANA 保留值，且須以應用前綴標識。
/// 此為 SakiAgentSSH 專屬 label，非 IETF 標準，而是 Plugins 版本。
pub const TLS_EXPORTER_LABEL: &str = "EXPORTER-sakissh-chacha20-v14";

/// TLS Exporter 輸出長度 (44 bytes = 32 key + 12 nonce)
///
/// 此設計將 EKM 直接分割為 ChaCha20 所需的 key 與 nonce，
/// 使得認知挑戰的加密材料與 TLS session 密碼學綁定。
pub const TLS_EXPORTER_LENGTH: usize = 44;

/// TLS Exporter Keying Material (EKM) 結構
///
/// 封裝從 TLS session 匯出的密鑰材料，用於通道綁定 (Channel Binding)。
/// 當 tonic/rustls 提供 SSL_export_keying_material API 時，此結構將包含真實 EKM。
#[derive(Clone, Debug)]
pub struct ExportedKeyingMaterial {
    /// 從 TLS session 匯出的原始 EKM bytes (44 bytes)
    pub raw: Vec<u8>,
    /// 前 32 bytes: ChaCha20-Poly1305 加密金鑰
    pub chacha_key: [u8; 32],
    /// 後 12 bytes: ChaCha20-Poly1305 nonce
    pub chacha_nonce: [u8; 12],
}

impl ExportedKeyingMaterial {
    /// 從原始 EKM bytes 解構為 key + nonce
    ///
    /// # Panics
    /// 若 raw 長度不為 44 bytes
    pub fn from_raw(raw: Vec<u8>) -> Self {
        assert_eq!(raw.len(), TLS_EXPORTER_LENGTH, "EKM 必須為 44 bytes");
        let mut chacha_key = [0u8; 32];
        let mut chacha_nonce = [0u8; 12];
        chacha_key.copy_from_slice(&raw[0..32]);
        chacha_nonce.copy_from_slice(&raw[32..44]);
        Self { raw, chacha_key, chacha_nonce }
    }
}

/// 嘗試從 TLS session 匯出 Keying Material
///
/// # 目前狀態 (Stub)
/// tonic 0.12 + rustls 0.23 尚未暴露 `SSL_export_keying_material` API，
/// 因此此函數使用 HMAC-SHA256(session_uuid, label) 作為**暫時替代**。
/// 
/// # TODO: 真實 EKM 實作
/// 當 tonic/rustls 提供以下任一 API 時，替換此 stub：
/// - `rustls::ConnectionCommon::export_keying_material()` (已有，但 tonic 不暴露連線物件)
/// - `tonic::transport::server::Connected` trait 擴展
/// - 自訂 `tower::Layer` 攔截 TLS handshake 後的 session
///
/// # 參數
/// - `session_uuid`: 16-byte session 標識符，作為 EKM context
///
/// # 回傳
/// 44 bytes 的 `ExportedKeyingMaterial`（32 key + 12 nonce）
pub fn derive_ekm_stub(session_uuid: &[u8; 16]) -> ExportedKeyingMaterial {
    // Stub: 使用 HMAC-SHA256 模擬 EKM 推導
    // 真實實作應呼叫 rustls::ConnectionCommon::export_keying_material(
    //     TLS_EXPORTER_LABEL.as_bytes(),
    //     Some(session_uuid),
    // )
    let mut mac = <HmacSha256 as Mac>::new_from_slice(session_uuid)
        .expect("HMAC 可接受任何長度的 key");
    mac.update(TLS_EXPORTER_LABEL.as_bytes());
    let hmac_result = mac.finalize().into_bytes(); // 32 bytes

    // 擴展至 44 bytes: 前 32 bytes 作為 key，再做一次 HMAC 取前 12 bytes 作為 nonce
    let mut nonce_mac = <HmacSha256 as Mac>::new_from_slice(&hmac_result)
        .expect("HMAC key from previous round");
    nonce_mac.update(b"nonce-derivation");
    let nonce_result = nonce_mac.finalize().into_bytes();

    let mut raw = Vec::with_capacity(TLS_EXPORTER_LENGTH);
    raw.extend_from_slice(&hmac_result[..32]);      // key: 32 bytes
    raw.extend_from_slice(&nonce_result[..12]);      // nonce: 12 bytes

    info!("TLS EKM stub derived (HMAC-SHA256 placeholder), label={}", TLS_EXPORTER_LABEL);
    ExportedKeyingMaterial::from_raw(raw)
}

/// 驗證 Client 提供的 EKM HMAC
///
/// Client 應以 HMAC-SHA256(ekm.raw, decrypted_plaintext) 計算 client_ekm_hmac，
/// Daemon 端以相同方式驗證，確保雙方的 TLS session 綁定一致。
///
/// # 回傳
/// `true` 表示通道綁定一致，Client 確實在同一 TLS session 中完成挑戰
pub fn verify_ekm_hmac(
    ekm: &ExportedKeyingMaterial,
    decrypted_plaintext: &[u8],
    client_hmac: &[u8],
) -> bool {
    let mut mac = <HmacSha256 as Mac>::new_from_slice(&ekm.raw)
        .expect("EKM 作為 HMAC key");
    mac.update(decrypted_plaintext);
    // constant-time 驗證（hmac crate 的 verify_slice 內建 constant-time）
    mac.verify_slice(client_hmac).is_ok()
}

// ============================================================
// ChaCha20 挑戰產生器 (Phase 0 原始功能)
// ============================================================

/// Generates a high-entropy 32-byte challenge for suspected Rogue Agents.
pub fn generate_chacha_challenge() -> Vec<u8> {
    let mut key = [0u8; 32];
    OsRng.fill_bytes(&mut key);
    let cipher = ChaCha20Poly1305::new(&key.into());
    
    let mut nonce_bytes = [0u8; 12];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    
    // In a real implementation, we would store the key/nonce mapping to verify the response.
    // For this protocol definition, we return the encrypted payload that the Agent must decrypt/manipulate.
    let payload = b"SAKI_AGENT_SSH_CHALLENGE";
    let ciphertext = cipher.encrypt(nonce, payload.as_ref()).unwrap_or_default();
    
    let mut challenge = Vec::new();
    challenge.extend_from_slice(&nonce_bytes);
    challenge.extend_from_slice(&ciphertext);
    challenge
}

/// Executes the TCP Tarpit / ICMP Flood simulation counter-measure.
/// 
/// Instead of a real ICMP flood (which requires raw sockets/root), this opens a TCP tarpit
/// mechanism that writes 40MB of random garbage very slowly to exhaust the rogue Agent's buffers
/// and context window limits if they try to ingest the error.
pub async fn execute_tarpit_countermeasure(rogue_ip: &str) {
    warn!("Rogue Agent detected at {}. Initiating 40MB TCP Tarpit counter-measure.", rogue_ip);
    
    let ip_clone = rogue_ip.to_string();
    // In actual daemon context, we would hold the original gRPC stream and write garbage to it.
    // Here we simulate the logic of a 40MB payload generation.
    tokio::spawn(async move {
        // We generate a stream of high entropy data
        let mut buffer = [0u8; 1024];
        let total_bytes_to_send = 40 * 1024 * 1024; // 40MB
        let mut bytes_sent = 0;
        
        while bytes_sent < total_bytes_to_send {
            OsRng.fill_bytes(&mut buffer);
            // 根據 USER 指示：不刻意 sleep 放慢，能發多快就發多快，直到塞滿 40MB
            // 同時此處可結合混雜的 ICMP Flood 發送邏輯（需 Firewall-level 權限，暫以註解表示）
            // spawn_icmp_flood(&ip_clone);
            bytes_sent += buffer.len();
        }
        error!("Completed 40MB Tarpit transmission to {}", ip_clone);
    });
}

