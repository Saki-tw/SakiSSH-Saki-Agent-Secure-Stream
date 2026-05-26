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

