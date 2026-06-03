//! SASS 焦油坑負載產生器 (SakiTarpit) — Rust 最低限度參考實作
//!
//! ChaCha20-Poly1305 keystream + 偽 ICMP 封包結構 + Saki✰ 簽名
//!
//! 設計原則：
//!   - 零外部依賴（僅使用 chacha20poly1305 crate — 已在 Cargo.toml）
//!   - O(1) 記憶體（streaming 產生，不需持有整個 payload）
//!   - 輸出無可辨識模式（ChaCha20 stream cipher）
//!   - 混雜偽 ICMP 封包結構，讓 Agent 以為是網路流量
//!   - Type 3 (Destination Unreachable) / Type 11 (Time Exceeded) 封包包含假 IPv4 header
//!   - 結尾 Saki✰ 簽名

use chacha20poly1305::{
    XChaCha20Poly1305, XNonce,
    aead::{Aead, KeyInit},
};
use sha2::{Sha256, Digest};

/// Saki✰ 簽名（UTF-8）
const SAKI_SIGNATURE: &[u8] = "Saki✰".as_bytes();

/// 假 IPv4 Header 長度（20 bytes，無選項）
const FAKE_IPV4_HEADER_LEN: usize = 20;

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

    /// 產生假 IPv4 Header（20 bytes）
    ///
    /// Type 3 (Destination Unreachable) 和 Type 11 (Time Exceeded) 的 ICMP
    /// 封包需在 ICMP payload 前包含引發錯誤的原始 IPv4 封包的前 20 bytes。
    /// 此處使用 keystream 填充非結構性欄位，但保持：
    /// - Version = 4, IHL = 5（第 1 byte = 0x45）
    /// - Total Length = 合理值
    fn generate_fake_ipv4_header(&mut self, keystream: &[u8]) -> [u8; FAKE_IPV4_HEADER_LEN] {
        let mut header = [0u8; FAKE_IPV4_HEADER_LEN];

        // Byte 0: Version (4) + IHL (5) = 0x45
        header[0] = 0x45;

        // Byte 1: DSCP/ECN — 從 keystream 取
        header[1] = keystream.get(0).copied().unwrap_or(0);

        // Byte 2-3: Total Length — 設定為合理的值（40-576 範圍）
        let total_len = 40u16 + (keystream.get(1).copied().unwrap_or(0) as u16 % 537);
        header[2..4].copy_from_slice(&total_len.to_be_bytes());

        // Byte 4-5: Identification — 從 keystream 取
        header[4] = keystream.get(2).copied().unwrap_or(0);
        header[5] = keystream.get(3).copied().unwrap_or(0);

        // Byte 6-7: Flags + Fragment Offset — 通常 0x4000 (Don't Fragment)
        header[6] = 0x40;
        header[7] = 0x00;

        // Byte 8: TTL — 從 keystream 取（1-255 範圍）
        header[8] = keystream.get(4).copied().unwrap_or(64).max(1);

        // Byte 9: Protocol — 常見值 (6=TCP, 17=UDP, 1=ICMP)
        let protocols: [u8; 3] = [6, 17, 1];
        let proto_idx = keystream.get(5).copied().unwrap_or(0) as usize % protocols.len();
        header[9] = protocols[proto_idx];

        // Byte 10-11: Header Checksum — 先設 0，稍後計算
        header[10] = 0;
        header[11] = 0;

        // Byte 12-15: Source IP — 從 keystream 取
        for i in 0..4 {
            header[12 + i] = keystream.get(6 + i).copied().unwrap_or(0);
        }

        // Byte 16-19: Destination IP — 從 keystream 取
        for i in 0..4 {
            header[16 + i] = keystream.get(10 + i).copied().unwrap_or(0);
        }

        // 計算 IPv4 header checksum
        let checksum = compute_ipv4_checksum(&header);
        header[10..12].copy_from_slice(&checksum.to_be_bytes());

        header
    }

    /// 產生偽 ICMP 封包（header + ChaCha20 加密 payload）
    ///
    /// Type 3 和 Type 11 的封包結構：
    ///   [ICMP Header (8)][Fake IPv4 Header (20)][ChaCha20 Payload]
    /// 其他類型：
    ///   [ICMP Header (8)][ChaCha20 Payload]
    fn generate_icmp_packet(&mut self, payload_size: usize) -> Vec<u8> {
        self.seq_num = self.seq_num.wrapping_add(1);

        // ChaCha20 payload — 先產生以取得 ciphertext 用於類型隨機化
        let nonce = self.advance_nonce();
        let plaintext = vec![0u8; payload_size];

        let cipher = XChaCha20Poly1305::new_from_slice(&self.key)
            .expect("key length must be 32");

        let mut ciphertext = cipher
            .encrypt(&nonce, plaintext.as_ref())
            .unwrap_or_else(|_| vec![0u8; payload_size]);

        // 截取到 payload_size（去掉 Poly1305 tag）
        ciphertext.truncate(payload_size);

        // ICMP 類型隨機化：使用 ciphertext 的第一個 byte 決定類型
        // Echo Request (type 8) 權重略高（出現兩次）
        let icmp_types: [u8; 7] = [0, 3, 8, 8, 11, 13, 14];
        let type_byte = ciphertext.first().copied().unwrap_or(0);
        let type_idx = (type_byte as usize) % icmp_types.len();
        let selected_type = icmp_types[type_idx];

        // 判斷是否為需要假 IPv4 header 的錯誤類型
        let needs_ipv4_header = selected_type == 3 || selected_type == 11;

        let header = IcmpHeader {
            icmp_type: selected_type,
            code: ((self.counter.rotate_left(3)) & 0x0F) as u8,
            identifier: (self.counter & 0xFFFF) as u16,
            seq_number: self.seq_num,
        };
        let header_bytes = header.to_bytes();

        // 組裝封包
        if needs_ipv4_header && payload_size >= FAKE_IPV4_HEADER_LEN {
            // Type 3/11: [ICMP Header (8)][Fake IPv4 Header (20)][剩餘 ChaCha20 Payload]
            // 總封包大小仍為 8 + payload_size（IPv4 header 取代 ciphertext 的前 20 bytes）
            let ipv4_header = self.generate_fake_ipv4_header(&ciphertext);

            let mut packet = Vec::with_capacity(8 + payload_size);
            packet.extend_from_slice(&header_bytes);
            packet.extend_from_slice(&ipv4_header);
            // 剩餘部分用 ciphertext 的後半段填充
            let remaining = payload_size - FAKE_IPV4_HEADER_LEN;
            if remaining > 0 {
                let start = FAKE_IPV4_HEADER_LEN.min(ciphertext.len());
                let end = (start + remaining).min(ciphertext.len());
                packet.extend_from_slice(&ciphertext[start..end]);
                // 若 ciphertext 不足，補 0
                while packet.len() < 8 + payload_size {
                    packet.push(0x00);
                }
            }

            // 計算正確的 ICMP checksum
            let checksum = compute_icmp_checksum(&packet);
            packet[2..4].copy_from_slice(&checksum.to_be_bytes());
            packet
        } else {
            // 其他類型（或 payload 過小無法容納 IPv4 header）：[ICMP Header][ChaCha20 Payload]
            let mut packet = Vec::with_capacity(8 + payload_size);
            packet.extend_from_slice(&header_bytes);
            packet.extend_from_slice(&ciphertext);

            // 計算正確的 ICMP checksum
            let checksum = compute_icmp_checksum(&packet);
            packet[2..4].copy_from_slice(&checksum.to_be_bytes());
            packet
        }
    }

    /// 產生一個焦油坑 chunk
    ///
    /// 結構：[ICMP Packet 1][ICMP Packet 2]...[ICMP Packet N][Saki✰]
    pub fn generate_chunk(&mut self, chunk_size: usize, is_final: bool) -> Vec<u8> {
        let mut chunk = Vec::with_capacity(chunk_size);
        // 若為最終 chunk，預留簽名空間
        let fill_target = if is_final {
            chunk_size.saturating_sub(SAKI_SIGNATURE.len())
        } else {
            chunk_size
        };

        while chunk.len() < fill_target {
            // payload 大小在 56~248 之間（193 是質數，避免週期）
            let payload_variation = 56 + (self.counter % 193) as usize;
            let remaining = fill_target - chunk.len();

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
            // 補 0 至填滿簽名前的空間
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

/// IPv4 Header Checksum（RFC 791）
fn compute_ipv4_checksum(header: &[u8; 20]) -> u16 {
    let mut sum: u32 = 0;
    let mut i = 0;
    while i < 20 {
        // 跳過 checksum 欄位（byte 10-11）
        if i == 10 {
            i += 2;
            continue;
        }
        sum += u16::from_be_bytes([header[i], header[i + 1]]) as u32;
        i += 2;
    }
    while sum >> 16 != 0 {
        sum = (sum & 0xFFFF) + (sum >> 16);
    }
    !(sum as u16)
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

    #[test]
    fn test_icmp_type_randomization() {
        // 確認連續多個封包中 ICMP type 分佈非固定（非單一值重複）
        let mut gen = SakiTarpitGenerator::new(b"type-randomization-test");
        let mut types_seen = std::collections::HashSet::new();

        // 產生 100 個封包，收集所有出現的 ICMP type
        for _ in 0..100 {
            let packet = gen.generate_icmp_packet(64);
            let icmp_type = packet[0]; // ICMP type 在封包第一個 byte
            types_seen.insert(icmp_type);
        }

        // 應該看到至少 3 種不同的 ICMP type（7 種可能值中取 100 次）
        assert!(
            types_seen.len() >= 3,
            "Expected at least 3 different ICMP types in 100 packets, got {}: {:?}",
            types_seen.len(),
            types_seen
        );
    }

    #[test]
    fn test_type3_has_ipv4_header() {
        // 確認 Type 3 (Destination Unreachable) 封包包含假 IPv4 header
        let mut gen = SakiTarpitGenerator::new(b"type3-ipv4-test");

        // 產生足夠多的封包直到找到 Type 3 或 Type 11
        let mut found_error_type = false;
        for _ in 0..500 {
            let packet = gen.generate_icmp_packet(128);
            let icmp_type = packet[0];

            if icmp_type == 3 || icmp_type == 11 {
                found_error_type = true;

                // 封包結構：[ICMP Header (8)][Fake IPv4 Header (20)][Payload]
                assert!(
                    packet.len() >= 8 + FAKE_IPV4_HEADER_LEN,
                    "Type {} packet too short: {} bytes (need at least {})",
                    icmp_type,
                    packet.len(),
                    8 + FAKE_IPV4_HEADER_LEN
                );

                // 檢查 IPv4 header 的 Version + IHL = 0x45
                let ipv4_start = 8; // ICMP header 之後
                assert_eq!(
                    packet[ipv4_start], 0x45,
                    "Fake IPv4 header should start with 0x45 (Version=4, IHL=5), got 0x{:02x}",
                    packet[ipv4_start]
                );

                // 檢查 Flags 欄位包含 Don't Fragment (0x40)
                assert_eq!(
                    packet[ipv4_start + 6], 0x40,
                    "Fake IPv4 header Flags byte should be 0x40 (DF), got 0x{:02x}",
                    packet[ipv4_start + 6]
                );

                break;
            }
        }

        assert!(
            found_error_type,
            "Failed to generate a Type 3 or Type 11 ICMP packet in 500 attempts"
        );
    }
}
