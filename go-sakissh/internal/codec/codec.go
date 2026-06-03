package codec

import (
	"bytes"
	"encoding/base64"
	"fmt"
	"io"

	"github.com/klauspost/compress/zstd"
)

var (
	encoder *zstd.Encoder
	decoder *zstd.Decoder
)

func init() {
	var err error
	encoder, err = zstd.NewWriter(nil)
	if err != nil {
		panic(err)
	}
	decoder, err = zstd.NewReader(nil)
	if err != nil {
		panic(err)
	}
}

// EncodePayload takes raw data, compresses it with zstd, and encodes it as base64 string.
func EncodePayload(data []byte) string {
	compressed := encoder.EncodeAll(data, make([]byte, 0, len(data)))
	return base64.StdEncoding.EncodeToString(compressed)
}

// DecodePayload takes a base64 encoded and zstd compressed string, and decompresses it back to raw data with a 5MB safety gate.
func DecodePayload(encoded string) ([]byte, error) {
	compressed, err := base64.StdEncoding.DecodeString(encoded)
	if err != nil {
		return nil, err
	}

	// 使用 io.LimitReader 動態限制解壓縮大小，防堵 zip bomb
	dec, err := zstd.NewReader(bytes.NewReader(compressed))
	if err != nil {
		return nil, err
	}
	defer dec.Close()

	const MaxDecompressedPayload = 5 * 1024 * 1024 // 5MB Limit
	limitedReader := io.LimitReader(dec, MaxDecompressedPayload)

	var decompressed bytes.Buffer
	_, err = io.Copy(&decompressed, limitedReader)
	if err != nil {
		return nil, err
	}

	// 探測是否仍有未解壓數據
	var probe [1]byte
	n, _ := dec.Read(probe[:])
	if n > 0 {
		return nil, fmt.Errorf("security violation: payload exceeds decompression limit (5MB)")
	}

	return decompressed.Bytes(), nil
}

// EncodeStreamChunk — 串流區塊專用編碼：Zstd 壓縮 + Base64 編碼
// 對齊 Rust: codec::encode_stream_chunk()
//
// 確保 CJK 多位元組字元在 gRPC 傳輸中不被截斷或誤譯。
// 壓縮失敗時 fallback 為原始 Base64（無壓縮）。
func EncodeStreamChunk(data []byte) string {
	if len(data) == 0 {
		return ""
	}
	// 嘗試 Zstd 壓縮 + Base64
	encoded := EncodePayload(data)
	if encoded != "" {
		return encoded
	}
	// Fallback: 僅 Base64（無壓縮）
	return base64.StdEncoding.EncodeToString(data)
}

// DecodeStreamChunk — 串流區塊專用解碼：Base64 解碼 + Zstd 解壓縮
// 對齊 Rust: codec::decode_stream_chunk()
//
// 含 5MiB 安全門控，防禦 zip bomb 攻擊
func DecodeStreamChunk(encoded string) ([]byte, error) {
	if len(encoded) == 0 {
		return []byte{}, nil
	}
	return DecodePayload(encoded)
}

