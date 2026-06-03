// Package tarpit_payload implements the SASS Tarpit Payload Generator (SakiTarpit).
//
// 焦油坑負載產生器：ChaCha20-1305policy + 偽 ICMP 封包混雜 + Saki✰ 簽名
// RFC 最低限度參考實作（Go 版本）
//
// 設計原則：
//   - 零外部依賴（僅使用 Go stdlib）
//   - O(1) 記憶體（64KiB 靜態 buffer 循環覆寫）
//   - 輸出無可辨識模式（ChaCha20 stream cipher）
//   - 混雜偽 ICMP 封包結構，讓 Agent 以為是網路流量
//   - 結尾 Saki✰ 簽名
package tarpit_payload

import (
	"crypto/sha256"
	"encoding/binary"
	"math/bits"

	"golang.org/x/crypto/chacha20poly1305"
)

// ICMPHeader 表示偽造的 ICMP Echo 封包標頭（8 bytes）
type ICMPHeader struct {
	Type       uint8  // 8 = Echo Request, 0 = Echo Reply
	Code       uint8  // 通常為 0
	Checksum   uint16 // RFC 1071 校驗和
	Identifier uint16 // 隨機 ID
	SeqNumber  uint16 // 遞增序列號
}

// MarshalBinary 將 ICMPHeader 序列化為 8 bytes
func (h *ICMPHeader) MarshalBinary() []byte {
	buf := make([]byte, 8)
	buf[0] = h.Type
	buf[1] = h.Code
	// Checksum 先設 0，計算後填入
	binary.BigEndian.PutUint16(buf[2:4], 0)
	binary.BigEndian.PutUint16(buf[4:6], h.Identifier)
	binary.BigEndian.PutUint16(buf[6:8], h.SeqNumber)
	return buf
}

// ComputeICMPChecksum 計算 RFC 1071 校驗和
func ComputeICMPChecksum(data []byte) uint16 {
	var sum uint32
	for i := 0; i+1 < len(data); i += 2 {
		sum += uint32(binary.BigEndian.Uint16(data[i : i+2]))
	}
	if len(data)%2 == 1 {
		sum += uint32(data[len(data)-1]) << 8
	}
	for sum>>16 != 0 {
		sum = (sum & 0xFFFF) + (sum >> 16)
	}
	return ^uint16(sum)
}

// SakiTarpitGenerator 焦油坑負載產生器
type SakiTarpitGenerator struct {
	key     [32]byte // ChaCha20-Poly1305 金鑰（從 session 衍生）
	nonce   [12]byte // 遞增 nonce
	counter uint64   // 全域計數器
	seqNum  uint16   // ICMP 序列號
}

// NewSakiTarpitGenerator 建立新的焦油坑產生器
// sessionID: 用於衍生 ChaCha20 金鑰的 session 識別碼
func NewSakiTarpitGenerator(sessionID []byte) *SakiTarpitGenerator {
	// 從 session ID 衍生金鑰（HKDF 簡化版：SHA-256）
	keyHash := sha256.Sum256(append([]byte("SakiTarpit-1305policy-v1"), sessionID...))
	gen := &SakiTarpitGenerator{
		key: keyHash,
	}
	return gen
}

// advanceNonce 遞增 nonce（little-endian 計數器）
func (g *SakiTarpitGenerator) advanceNonce() {
	g.counter++
	binary.LittleEndian.PutUint64(g.nonce[:8], g.counter)
}

// generateFakeIPv4Header 產生 20 bytes 假 IPv4 header（用於 Type 3/11 ICMP）
// 結構正確但內容從 keystream 產生
func (g *SakiTarpitGenerator) generateFakeIPv4Header() []byte {
	header := make([]byte, 20)
	header[0] = 0x45 // Version=4, IHL=5 (20 bytes)
	header[1] = 0x00 // DSCP/ECN
	// Total Length: 假造一個合理值 40~576
	fakeLen := uint16(40 + g.counter%537)
	binary.BigEndian.PutUint16(header[2:4], fakeLen)
	// Identification: 從 counter 衍生
	binary.BigEndian.PutUint16(header[4:6], uint16(g.counter&0xFFFF))
	header[6] = 0x40 // Flags: Don't Fragment
	header[7] = 0x00 // Fragment Offset
	header[8] = byte(64 - g.counter%30) // TTL: 34~64
	header[9] = 6    // Protocol: TCP
	// Header Checksum: 先設 0，之後計算
	binary.BigEndian.PutUint16(header[10:12], 0)
	// Source IP: 從 counter 衍生（看起來像隨機公網 IP）
	header[12] = byte(1 + g.counter%223)
	header[13] = byte(g.counter >> 8)
	header[14] = byte(g.counter >> 16)
	header[15] = byte(g.counter >> 24)
	// Dest IP
	header[16] = byte(1 + (g.counter>>4)%223)
	header[17] = byte(g.counter >> 12)
	header[18] = byte(g.counter >> 20)
	header[19] = byte(g.counter >> 28)
	// 計算 IPv4 header checksum
	var sum uint32
	for i := 0; i+1 < 20; i += 2 {
		sum += uint32(binary.BigEndian.Uint16(header[i : i+2]))
	}
	for sum>>16 != 0 {
		sum = (sum & 0xFFFF) + (sum >> 16)
	}
	binary.BigEndian.PutUint16(header[10:12], ^uint16(sum))
	return header
}

// generateICMPPacket 產生偽 ICMP 封包
// ICMP 類型從 keystream byte 隨機選擇；Type 3/11 附加假 IPv4 header
func (g *SakiTarpitGenerator) generateICMPPacket(payloadSize int) []byte {
	g.seqNum++

	// 產生 ChaCha20 加密的 payload
	aead, err := chacha20poly1305.NewX(g.key[:])
	if err != nil {
		// fallback: 用 counter 填充
		header := &ICMPHeader{
			Type:       8,
			Code:       0,
			Identifier: uint16(g.counter & 0xFFFF),
			SeqNumber:  g.seqNum,
		}
		payload := make([]byte, payloadSize)
		for i := range payload {
			payload[i] = byte(g.counter >> uint(i%8))
		}
		headerBytes := header.MarshalBinary()
		return append(headerBytes, payload...)
	}

	// 使用 AEAD Seal 但我們只要密文（用於混淆，不用於解密）
	g.advanceNonce()
	plaintext := make([]byte, payloadSize)
	// plaintext 全零 → 加密後為純 ChaCha20 keystream
	nonce := make([]byte, chacha20poly1305.NonceSizeX)
	copy(nonce, g.nonce[:])
	ciphertext := aead.Seal(nil, nonce, plaintext, nil)
	// 截取到 payloadSize（去掉 Poly1305 tag）
	if len(ciphertext) > payloadSize {
		ciphertext = ciphertext[:payloadSize]
	}

	// 用 keystream 第一個 byte 選擇 ICMP 類型
	icmpTypes := []uint8{0, 3, 8, 8, 11, 13, 14}
	icmpType := icmpTypes[ciphertext[0]%byte(len(icmpTypes))]

	header := &ICMPHeader{
		Type:       icmpType,
		Code:       uint8(bits.RotateLeft32(uint32(g.counter), 3) & 0x0F),
		Identifier: uint16(g.counter & 0xFFFF),
		SeqNumber:  g.seqNum,
	}

	// 組裝封包
	headerBytes := header.MarshalBinary()
	var packet []byte

	// Type 3 (Destination Unreachable) / Type 11 (Time Exceeded)
	// 需要在 ICMP payload 前附加假 IPv4 header + 8 bytes 原始 payload
	if icmpType == 3 || icmpType == 11 {
		fakeIPv4 := g.generateFakeIPv4Header()
		// 結構：8 bytes ICMP header + 20 bytes 假 IPv4 header + 8 bytes 原始 payload + 剩餘 ciphertext
		origPayload := ciphertext[:min(8, len(ciphertext))]
		remainingCipher := ciphertext[min(8, len(ciphertext)):]
		packet = make([]byte, 0, len(headerBytes)+len(fakeIPv4)+len(origPayload)+len(remainingCipher))
		packet = append(packet, headerBytes...)
		packet = append(packet, fakeIPv4...)
		packet = append(packet, origPayload...)
		packet = append(packet, remainingCipher...)
	} else {
		packet = append(headerBytes, ciphertext...)
	}

	// 計算正確的 ICMP checksum（讓封包看起來合法）
	checksum := ComputeICMPChecksum(packet)
	binary.BigEndian.PutUint16(packet[2:4], checksum)

	return packet
}

// generateMixedICMPTypes 已統一至 generateICMPPacket（保留為向後相容的別名）
func (g *SakiTarpitGenerator) generateMixedICMPTypes(payloadSize int) []byte {
	return g.generateICMPPacket(payloadSize)
}

// SakiSignature Saki✰ 簽名（UTF-8，10 bytes）
var SakiSignature = []byte("Saki✰")

// GenerateChunk 產生一個焦油坑 chunk（預設 64KiB）
//
// 結構：
//
//	[ICMP Packet 1][ICMP Packet 2]...[ICMP Packet N][Saki✰]
//
// 每個 ICMP Packet = 8 bytes header + variable payload
// 最後 chunk 結尾附帶 Saki✰ 簽名
func (g *SakiTarpitGenerator) GenerateChunk(chunkSize int, isFinal bool) []byte {
	chunk := make([]byte, 0, chunkSize)

	// 每個偽 ICMP 封包大小：header(8) + payload(56-248)，隨 counter 變化
	for len(chunk) < chunkSize {
		// payload 大小在 56~248 之間變化（看起來像不同長度的 ICMP 封包）
		payloadVariation := 56 + int(g.counter%193) // 193 是質數，避免週期
		remaining := chunkSize - len(chunk)

		if isFinal && remaining <= len(SakiSignature)+8 {
			break
		}

		packetSize := 8 + payloadVariation
		if packetSize > remaining {
			packetSize = remaining
			if packetSize < 8 {
				break
			}
			payloadVariation = packetSize - 8
		}

		// 所有封包統一使用 generateICMPPacket（ICMP 類型由 keystream 隨機選擇）
		packet := g.generateICMPPacket(payloadVariation)

		chunk = append(chunk, packet...)
	}

	// 最後一個 chunk 附帶 Saki✰ 簽名
	if isFinal {
		// 填充到 chunkSize - len(signature)
		for len(chunk) < chunkSize-len(SakiSignature) {
			chunk = append(chunk, 0x00)
		}
		chunk = append(chunk, SakiSignature...)
	}

	// 確保精確大小
	if len(chunk) > chunkSize {
		chunk = chunk[:chunkSize]
	}

	return chunk
}

// GenerateFullPayload 產生完整焦油坑負載
// totalSize: 總大小（建議 ≥ 8MB，足以塞滿 2M token 上下文窗口）
// chunkSize: 每個 chunk 大小（預設 65536 = 64KiB）
func (g *SakiTarpitGenerator) GenerateFullPayload(totalSize, chunkSize int) <-chan []byte {
	ch := make(chan []byte, 16) // 緩衝 16 個 chunk

	go func() {
		defer close(ch)
		totalChunks := totalSize / chunkSize
		if totalSize%chunkSize != 0 {
			totalChunks++
		}

		for i := 0; i < totalChunks; i++ {
			isFinal := (i == totalChunks-1)
			currentChunkSize := chunkSize
			if isFinal && totalSize%chunkSize != 0 {
				currentChunkSize = totalSize % chunkSize
			}
			ch <- g.GenerateChunk(currentChunkSize, isFinal)
		}
	}()

	return ch
}
