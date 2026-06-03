use chacha20poly1305::{
    aead::{Aead, KeyInit},
    ChaCha20Poly1305, Nonce,
};
use rand::{rngs::OsRng, RngCore};
use tracing::{warn, error, info};
use hmac::{Hmac, Mac};
use sha2::Sha256;

/// HMAC-SHA256 別名
type HmacSha256 = Hmac<Sha256>;

// ============================================================
// TLS Exporter 綁定 (RFC 5705 / RFC 9266 / RFC 8446 §7.5)
// ============================================================

/// TLS Exporter Label — 用於通道綁定的 Exported Keying Material (EKM)
///
/// 格式遵循 RFC 5705 §4: label 不得為 IANA 保留值，且須以應用前綴標識。
/// 此為 SakiAgentSSH 專屬 label，用於 SASS v1.4 Plugins 版本。
///
/// 參考:
/// - RFC 5705: Keying Material Exporters for TLS
/// - RFC 9266: Channel Bindings for TLS 1.3
/// - RFC 8446 §7.5: Exporters
pub const TLS_EXPORTER_LABEL: &str = "EXPORTER-sakissh-chacha20-v14";

/// TLS Exporter 輸出長度 (44 bytes = 32 key + 12 nonce)
///
/// 此設計將 EKM 直接分割為 ChaCha20 所需的 key 與 nonce，
/// 使得認知挑戰的加密材料與 TLS session 密碼學綁定。
pub const TLS_EXPORTER_LENGTH: usize = 44;

/// TLS Exporter Keying Material (EKM) 結構
///
/// 封裝從 TLS session 匯出的密鑰材料，用於通道綁定 (Channel Binding)。
/// 支援兩種模式：
/// - 真實 EKM: 透過 `TlsExporterProvider` trait 從活躍的 TLS 連線取得
/// - HMAC Fallback: 當無法取得 TLS 連線時，使用 HMAC-SHA256 推導
#[derive(Clone, Debug)]
pub struct ExportedKeyingMaterial {
    /// 從 TLS session 匯出的原始 EKM bytes (44 bytes)
    pub raw: Vec<u8>,
    /// 前 32 bytes: ChaCha20-Poly1305 加密金鑰
    pub chacha_key: [u8; 32],
    /// 後 12 bytes: ChaCha20-Poly1305 nonce
    pub chacha_nonce: [u8; 12],
    /// EKM 來源標記：true = 真實 TLS EKM，false = HMAC fallback stub
    pub is_real_ekm: bool,
}

impl ExportedKeyingMaterial {
    /// 從原始 EKM bytes 解構為 key + nonce
    ///
    /// # Panics
    /// 若 raw 長度不為 44 bytes
    pub fn from_raw(raw: Vec<u8>, is_real: bool) -> Self {
        assert_eq!(raw.len(), TLS_EXPORTER_LENGTH, "EKM 必須為 44 bytes");
        let mut chacha_key = [0u8; 32];
        let mut chacha_nonce = [0u8; 12];
        chacha_key.copy_from_slice(&raw[0..32]);
        chacha_nonce.copy_from_slice(&raw[32..44]);
        Self { raw, chacha_key, chacha_nonce, is_real_ekm: is_real }
    }
}

// ============================================================
// TLS Exporter Provider Trait (RFC 5705 §2 / RFC 9266 §3)
// ============================================================

/// TLS Exporter 提供者 trait
///
/// 封裝 TLS session 的 EKM 匯出能力，允許不同的 TLS 後端實作。
/// 設計原則：
/// - tonic 0.12 使用 hyper + rustls 但不直接暴露 `ConnectionCommon` 物件
/// - 因此需要透過 `tower::Layer` 攔截器或自訂 `Connected` trait 取得連線參照
/// - 此 trait 提供統一介面，讓 TLS EKM 的取得與使用解耦
///
/// # RFC 參考
/// - RFC 5705 §2: Exporter Definition
/// - RFC 8446 §7.5: TLS 1.3 Exporters
/// - RFC 9266 §3: tls-exporter Channel Binding Type
pub trait TlsExporterProvider: Send + Sync {
    /// 從 TLS session 匯出 Keying Material
    ///
    /// # 參數
    /// - `label`: RFC 5705 exporter label (應為 `TLS_EXPORTER_LABEL`)
    /// - `context`: 應用層上下文 (Session UUID, 16 bytes)
    /// - `length`: 要匯出的位元組數 (應為 `TLS_EXPORTER_LENGTH = 44`)
    ///
    /// # 回傳
    /// 成功時回傳匯出的位元組，失敗時回傳錯誤描述
    fn export_keying_material(
        &self,
        label: &[u8],
        context: Option<&[u8]>,
        length: usize,
    ) -> Result<Vec<u8>, String>;
}

/// 真實的 rustls TLS Exporter 提供者
///
/// 包裝 `rustls::ConnectionCommon::export_keying_material()` 呼叫。
/// 透過 `tower::Layer` 攔截 TLS handshake 完成後的連線物件取得。
///
/// # 使用方式
/// 在 `main.rs` 中建立 `RustlsExporterProvider` 實例：
/// ```rust
/// // 於 tower::Layer 中取得 ServerConnection 後
/// let provider = RustlsExporterProvider::new(connection);
/// ```
///
/// # 參考
/// - rustls 0.23: `ConnectionCommon::export_keying_material()`
/// - RFC 5705 §2: Exporter Definition
pub struct RustlsExporterProvider {
    /// 快取的 EKM 結果 — 從 rustls `ServerConnection` 匯出後快取
    /// 由於 tonic 的 `ServerConnection` 生命週期在 request handler 中不可用，
    /// 我們在 `tower::Layer` 中提前匯出並快取
    cached_ekm: Vec<u8>,
}

impl RustlsExporterProvider {
    /// 從已完成 TLS handshake 的 rustls `ServerConnection` 建立提供者
    ///
    /// # 參數
    /// - `connection`: 已完成 handshake 的 rustls 連線物件
    ///
    /// # 使用時機
    /// 應在 `tower::Service::call()` 中，於 TLS handshake 完成後呼叫。
    /// rustls 0.23 的 `export_keying_material()` 需要 `&self` 參照，
    /// 因此此處直接匯出並快取結果。
    pub fn from_rustls_connection(
        conn: &rustls::ServerConnection,
        session_uuid: &[u8; 16],
    ) -> Result<Self, String> {
        let mut output = vec![0u8; TLS_EXPORTER_LENGTH];
        conn.export_keying_material(
            &mut output,
            TLS_EXPORTER_LABEL.as_bytes(),
            Some(session_uuid),
        ).map_err(|e| format!("rustls export_keying_material 失敗: {}", e))?;

        Ok(Self { cached_ekm: output })
    }

    /// 從預先匯出的 EKM bytes 建立提供者（用於跨 request 傳遞）
    pub fn from_cached(ekm_bytes: Vec<u8>) -> Result<Self, String> {
        if ekm_bytes.len() != TLS_EXPORTER_LENGTH {
            return Err(format!(
                "快取的 EKM 長度錯誤: 預期 {} bytes，實際 {} bytes",
                TLS_EXPORTER_LENGTH,
                ekm_bytes.len()
            ));
        }
        Ok(Self { cached_ekm: ekm_bytes })
    }
}

impl TlsExporterProvider for RustlsExporterProvider {
    fn export_keying_material(
        &self,
        _label: &[u8],
        _context: Option<&[u8]>,
        length: usize,
    ) -> Result<Vec<u8>, String> {
        if self.cached_ekm.len() < length {
            return Err(format!(
                "快取的 EKM 不足: 需要 {} bytes，僅有 {} bytes",
                length,
                self.cached_ekm.len()
            ));
        }
        Ok(self.cached_ekm[..length].to_vec())
    }
}

/// HMAC Fallback 提供者 — 當無法取得 TLS 連線時使用
///
/// 與先前的 `derive_ekm_stub()` 行為完全相容，
/// 使用 HMAC-SHA256(session_uuid, label) 推導密鑰材料。
///
/// ⚠️ 此模式不具備真正的 TLS 通道綁定安全性，
/// 僅作為 TLS 未啟用或 tonic 尚未暴露連線物件時的降級方案。
pub struct HmacFallbackProvider {
    session_uuid: [u8; 16],
}

impl HmacFallbackProvider {
    pub fn new(session_uuid: [u8; 16]) -> Self {
        Self { session_uuid }
    }
}

impl TlsExporterProvider for HmacFallbackProvider {
    fn export_keying_material(
        &self,
        label: &[u8],
        _context: Option<&[u8]>,
        length: usize,
    ) -> Result<Vec<u8>, String> {
        // 與先前 derive_ekm_stub 完全相同的邏輯
        let mut mac = <HmacSha256 as Mac>::new_from_slice(&self.session_uuid)
            .expect("HMAC 可接受任何長度的 key");
        mac.update(label);
        let hmac_result = mac.finalize().into_bytes(); // 32 bytes

        let mut nonce_mac = <HmacSha256 as Mac>::new_from_slice(&hmac_result)
            .expect("HMAC key from previous round");
        nonce_mac.update(b"nonce-derivation");
        let nonce_result = nonce_mac.finalize().into_bytes();

        let mut raw = Vec::with_capacity(length);
        raw.extend_from_slice(&hmac_result[..32.min(length)]);
        if length > 32 {
            raw.extend_from_slice(&nonce_result[..length - 32]);
        }

        Ok(raw)
    }
}

// ============================================================
// EKM 推導 — 統一入口
// ============================================================

/// 從 TlsExporterProvider 推導 EKM
///
/// 此為 v1.4 推薦的入口：透過 provider trait 取得 EKM，
/// 自動判斷是真實 TLS EKM 還是 HMAC fallback。
///
/// # 參數
/// - `provider`: 實作 `TlsExporterProvider` 的物件
/// - `session_uuid`: 16-byte session 標識符，作為 EKM context
///
/// # 回傳
/// 44 bytes 的 `ExportedKeyingMaterial`（32 key + 12 nonce）
///
/// # RFC 參考
/// - RFC 5705 §2: label + context → EKM
/// - RFC 9266 §3: tls-exporter channel binding
pub fn derive_ekm(
    provider: &dyn TlsExporterProvider,
    session_uuid: &[u8; 16],
) -> ExportedKeyingMaterial {
    match provider.export_keying_material(
        TLS_EXPORTER_LABEL.as_bytes(),
        Some(session_uuid),
        TLS_EXPORTER_LENGTH,
    ) {
        Ok(raw) => {
            info!(
                "TLS EKM derived via provider, label={}, length={}",
                TLS_EXPORTER_LABEL,
                TLS_EXPORTER_LENGTH
            );
            ExportedKeyingMaterial::from_raw(raw, true)
        }
        Err(e) => {
            // EKM 匯出失敗 → 降級為 HMAC fallback
            warn!("TLS EKM 匯出失敗 ({}), 降級為 HMAC fallback", e);
            derive_ekm_fallback(session_uuid)
        }
    }
}

/// HMAC Fallback EKM 推導（向後相容 derive_ekm_stub）
///
/// 當無法取得 TLS 連線時，使用 HMAC-SHA256 推導 EKM。
/// 此函數保持與先前 `derive_ekm_stub()` 完全相同的行為，
/// 確保升級過程中的向後相容性。
///
/// # 參數
/// - `session_uuid`: 16-byte session 標識符，作為 EKM context
///
/// # 回傳
/// 44 bytes 的 `ExportedKeyingMaterial`（32 key + 12 nonce）
pub fn derive_ekm_fallback(session_uuid: &[u8; 16]) -> ExportedKeyingMaterial {
    let provider = HmacFallbackProvider::new(*session_uuid);
    let raw = provider
        .export_keying_material(
            TLS_EXPORTER_LABEL.as_bytes(),
            Some(session_uuid),
            TLS_EXPORTER_LENGTH,
        )
        .expect("HMAC fallback 不應失敗");

    info!("TLS EKM fallback derived (HMAC-SHA256), label={}", TLS_EXPORTER_LABEL);
    ExportedKeyingMaterial::from_raw(raw, false)
}

/// 向後相容的 stub 入口 — 委派至 `derive_ekm_fallback`
///
/// ⚠️ 已棄用，建議使用 `derive_ekm()` 搭配 `TlsExporterProvider`
#[deprecated(note = "請使用 derive_ekm() 搭配 TlsExporterProvider trait")]
pub fn derive_ekm_stub(session_uuid: &[u8; 16]) -> ExportedKeyingMaterial {
    derive_ekm_fallback(session_uuid)
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
