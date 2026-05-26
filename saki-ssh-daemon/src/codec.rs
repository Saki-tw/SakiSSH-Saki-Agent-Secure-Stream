use std::io::Read;
use zstd::stream::{read::Decoder, write::Encoder};
use base64::{Engine as _, engine::general_purpose::STANDARD};

/// Encodes a raw binary payload by compressing it with zstd and then encoding as base64.
/// This completely bypasses any CJK translation issues during transmission.
pub fn encode_payload(raw_data: &[u8]) -> Result<Vec<u8>, std::io::Error> {
    let mut compressed = Vec::new();
    {
        let mut encoder = Encoder::new(&mut compressed, 3)?; // Level 3 is a good default
        std::io::Write::write_all(&mut encoder, raw_data)?;
        encoder.finish()?;
    }
    let b64_string = STANDARD.encode(&compressed);
    Ok(b64_string.into_bytes())
}

/// Decodes a base64-encoded, zstd-compressed payload back to its raw binary form.
pub fn decode_payload(encoded_data: &[u8]) -> Result<Vec<u8>, std::io::Error> {
    let compressed = STANDARD.decode(encoded_data)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, format!("Base64 decode error: {}", e)))?;
    
    let mut decoder = Decoder::new(compressed.as_slice())?;
    let mut decompressed = Vec::new();
    decoder.read_to_end(&mut decompressed)?;
    Ok(decompressed)
}

/// 串流區塊專用編碼：Zstd 壓縮 + Base64 編碼
/// 確保 CJK 多位元組字元在 gRPC 傳輸中不被截斷或誤譯
pub fn encode_stream_chunk(data: &[u8]) -> Vec<u8> {
    // 空資料直接返回，避免不必要的壓縮開銷
    if data.is_empty() {
        return Vec::new();
    }
    // 壓縮失敗時 fallback 為原始 Base64（無壓縮）
    match encode_payload(data) {
        Ok(encoded) => encoded,
        Err(_) => {
            use base64::{Engine as _, engine::general_purpose::STANDARD};
            STANDARD.encode(data).into_bytes()
        }
    }
}

/// 串流區塊專用解碼：Base64 解碼 + Zstd 解壓縮
/// 含 5MiB 安全門控，防禦 zip bomb 攻擊
const MAX_DECODED_SIZE: usize = 5 * 1024 * 1024; // 5MiB

pub fn decode_stream_chunk(encoded: &[u8]) -> Result<Vec<u8>, String> {
    if encoded.is_empty() {
        return Ok(Vec::new());
    }
    let decoded = decode_payload(encoded)
        .map_err(|e| format!("串流區塊解碼失敗: {}", e))?;
    if decoded.len() > MAX_DECODED_SIZE {
        return Err(format!(
            "解壓後大小 {} 超過安全上限 {} bytes (5MiB)，可能為 zip bomb",
            decoded.len(),
            MAX_DECODED_SIZE
        ));
    }
    Ok(decoded)
}
