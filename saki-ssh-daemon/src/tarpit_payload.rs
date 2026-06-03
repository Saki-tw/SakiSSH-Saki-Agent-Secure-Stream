//! SASS 焦油坑負載產生器 (SakiTarpit) — Rust 最低限度參考實作
//!
//! ChaCha20-Poly1305 keystream + 偽 ICMP 封包結構 + Saki✰ 簽名
//!
//! 設計原則：
//!   - 零外部依賴（僅使用 chacha20poly1305 crate — 已在 Cargo.toml）
//!   - O(1) 記憶體（streaming 產生，不需持有整個 payload）
//!   - 輸出無可辨識模式（ChaCha20 stream cipher）
//!   - 混雜偽 ICMP 封包結構，讓 Agent 以為是網路流量
//!   - 結尾 Saki✰ 簽名

use chacha20poly1305::{
    XChaCha20Poly1305, XNonce,
    aead::{Aead, KeyInit},
};
use sha2::{Sha256, Digest};

/// Saki✰ 簽名（UTF-8）
const SAKI_SIGNATURE: &[u8] = "Saki✰".as_bytes();

/// ICMP 封包標頭（8 bytes）
struct IcmpHeader {
    icmp_type: u8,
    code: u8,
    identifier: u16,
    seq_number: u16,
}

impl IcmpHeader {
    fn to_bytes(&self) -> [u8; 8] {
        let mut buf = [0u8; 8];
        buf[0] = self.icmp_type;
        buf[1] = self.code;
        // checksum at [2:4] — 先設 0，稍後計算
        buf[4..6].copy_from_slice(&self.identifier.to_be_bytes());
        buf[6..8].copy_from_slice(&self.seq_number.to_be_bytes());
        buf
    }
}

/// RFC 1071 校驗和
fn compute_icmp_checksum(data: &[u8]) -> u16 {
    let mut sum: u32 = 0;
    let mut i = 0;
    while i + 1 < data.len() {
        sum += u16::from_be_bytes([data[i], data[i + 1]]) as u32;
        i += 2;
    }
    if data.len() % 2 == 1 {
        sum += (data[data.len() - 1] as u32) << 8;
    }
    while sum >> 16 != 0 {
        sum = (sum & 0xFFFF) + (sum >> 16);
    }
    !(sum as u16)
}

/// 焦油坑負載產生器
pub struct SakiTarpitGenerator {
    key: [u8; 32],
    counter: u64,
    seq_num: u16,
}

impl SakiTarpitGenerator {
    /// 建立新的焦油坑產生器
    /// session_id: 用於衍生 ChaCha20 金鑰的 session 識別碼
    pub fn new(session_id: &[u8]) -> Self {
        // 從 session ID 衍生金鑰（SHA-256）
        let mut hasher = Sha256::new();
        hasher.update(b"SakiTarpit-1305policy-v1");
        hasher.update(session_id);
        let key: [u8; 32] = hasher.finalize().into();

        Self {
            key,
            counter: 0,
            seq_num: 0,
        }
    }

    /// 遞增 nonce 並回傳 24-byte XNonce
    fn advance_nonce(&mut self) -> XNonce {
        self.counter += 1;
        let mut nonce = [0u8; 24];
        nonce[..8].copy_from_slice(&self.counter.to_le_bytes());
        *XNonce::from_slice(&nonce)
    }

    /// 產生偽 ICMP 封包（header + ChaCha20 加密 payload）
    fn generate_icmp_packet(&mut self, payload_size: usize) -> Vec<u8> {
        self.seq_num = self.seq_num.wrapping_add(1);

        // 根據 counter 選擇 ICMP 類型
        let icmp_types: [u8; 6] = [0, 3, 8, 11, 13, 14];
        let type_idx = (self.counter % icmp_types.len() as u64) as usize;

        let header = IcmpHeader {
            icmp_type: icmp_types[type_idx],
            code: ((self.counter.rotate_left(3)) & 0x0F) as u8,
            identifier: (self.counter & 0xFFFF) as u16,
            seq_number: self.seq_num,
        };
        let header_bytes = header.to_bytes();

        // ChaCha20 payload
        let nonce = self.advance_nonce();
        let plaintext = vec![0u8; payload_size];

        let cipher = XChaCha20Poly1305::new_from_slice(&self.key)
            .expect("key length must be 32");

        let mut ciphertext = cipher
            .encrypt(&nonce, plaintext.as_ref())
            .unwrap_or_else(|_| vec![0u8; payload_size]);

        // 截取到 payload_size（去掉 Poly1305 tag）
        ciphertext.truncate(payload_size);

        // 組裝封包
        let mut packet = Vec::with_capacity(8 + payload_size);
        packet.extend_from_slice(&header_bytes);
        packet.extend_from_slice(&ciphertext);

        // 計算正確的 ICMP checksum
        let checksum = compute_icmp_checksum(&packet);
        packet[2..4].copy_from_slice(&checksum.to_be_bytes());

        packet
    }

    /// 產生一個焦油坑 chunk
    ///
    /// 結構：[ICMP Packet 1][ICMP Packet 2]...[ICMP Packet N][Saki✰]
    pub fn generate_chunk(&mut self, chunk_size: usize, is_final: bool) -> Vec<u8> {
        let mut chunk = Vec::with_capacity(chunk_size);

        while chunk.len() < chunk_size {
            // payload 大小在 56~248 之間（193 是質數，避免週期）
            let payload_variation = 56 + (self.counter % 193) as usize;
            let remaining = chunk_size - chunk.len();

            if is_final && remaining <= SAKI_SIGNATURE.len() + 8 {
                break;
            }

            let mut packet_size = 8 + payload_variation;
            if packet_size > remaining {
                packet_size = remaining;
                if packet_size < 8 {
                    break;
                }
            }

            let packet = self.generate_icmp_packet(packet_size - 8);
            chunk.extend_from_slice(&packet);
        }

        // 最後一個 chunk 附帶 Saki✰ 簽名
        if is_final {
            while chunk.len() < chunk_size.saturating_sub(SAKI_SIGNATURE.len()) {
                chunk.push(0x00);
            }
            chunk.extend_from_slice(SAKI_SIGNATURE);
        }

        chunk.truncate(chunk_size);
        chunk
    }

    /// 產生完整焦油坑負載的迭代器
    pub fn generate_full_payload(
        &mut self,
        total_size: usize,
        chunk_size: usize,
    ) -> Vec<Vec<u8>> {
        let mut chunks = Vec::new();
        let total_chunks = (total_size + chunk_size - 1) / chunk_size;

        for i in 0..total_chunks {
            let is_final = i == total_chunks - 1;
            let current_chunk_size = if is_final && total_size % chunk_size != 0 {
                total_size % chunk_size
            } else {
                chunk_size
            };
            chunks.push(self.generate_chunk(current_chunk_size, is_final));
        }

        chunks
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_icmp_checksum() {
        // RFC 1071 測試向量
        let data = [0x08, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x01];
        let checksum = compute_icmp_checksum(&data);
        assert_ne!(checksum, 0); // 非零校驗和
    }

    #[test]
    fn test_generate_chunk_size() {
        let mut gen = SakiTarpitGenerator::new(b"test-session-123");
        let chunk = gen.generate_chunk(65536, false);
        assert_eq!(chunk.len(), 65536);
    }

    #[test]
    fn test_final_chunk_has_signature() {
        let mut gen = SakiTarpitGenerator::new(b"test-session-123");
        let chunk = gen.generate_chunk(1024, true);
        assert!(chunk.ends_with(SAKI_SIGNATURE));
    }

    #[test]
    fn test_no_pattern_in_output() {
        let mut gen = SakiTarpitGenerator::new(b"test-session-123");
        let chunk1 = gen.generate_chunk(4096, false);
        let chunk2 = gen.generate_chunk(4096, false);
        // 兩個 chunk 不應完全相同
        assert_ne!(chunk1, chunk2);
    }

    #[test]
    fn test_different_sessions_different_output() {
        let mut gen1 = SakiTarpitGenerator::new(b"session-A");
        let mut gen2 = SakiTarpitGenerator::new(b"session-B");
        let chunk1 = gen1.generate_chunk(4096, false);
        let chunk2 = gen2.generate_chunk(4096, false);
        assert_ne!(chunk1, chunk2);
    }

    #[test]
    fn test_full_payload_total_size() {
        let mut gen = SakiTarpitGenerator::new(b"test-session");
        let chunks = gen.generate_full_payload(256 * 1024, 65536); // 256KB
        let total: usize = chunks.iter().map(|c| c.len()).sum();
        assert_eq!(total, 256 * 1024);
    }
}
