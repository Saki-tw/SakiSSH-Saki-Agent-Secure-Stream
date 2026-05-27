// =============================================================================
// Package defense — tls_exporter.go
// SASS Plugin #2: TLS Exporter Binding for Cognitive Challenge (Go 實作)
//
// 對應 Rust: threat_defense.rs (TlsExporterProvider / derive_ekm / verify_ekm_hmac)
// RFC 參考:
//   - RFC 5705: Keying Material Exporters for TLS
//   - RFC 9266: Channel Bindings for TLS 1.3
//   - RFC 8446 §7.5: Exporters
//   - draft-sakistudio-sass-00 Appendix C.2 (anchor: tls-exporter-binding)
//
// TLS Exporter Label: "EXPORTER-sakissh-chacha20-v14"
// Context: Session UUID (16 bytes)
// Length: 44 bytes (32-byte ChaCha20 key + 12-byte nonce)
//
// Copyright (c) 2026 Saki Studio. All rights reserved.
// =============================================================================

package defense

import (
	"crypto/hmac"
	"crypto/sha256"
	"crypto/subtle"
	"crypto/tls"
	"fmt"
	"log"
)

// TLSExporterLabel — RFC 5705 §4 格式的 TLS Exporter Label
// 對齊 Rust: TLS_EXPORTER_LABEL 常量
const TLSExporterLabel = "EXPORTER-sakissh-chacha20-v14"

// TLSExporterLength — TLS Exporter 輸出長度 (44 bytes = 32 key + 12 nonce)
// 對齊 Rust: TLS_EXPORTER_LENGTH = 44
const TLSExporterLength = 44

// SessionUUIDLength — Session UUID 長度 (16 bytes)
const SessionUUIDLength = 16

// ExportedKeyingMaterial 封裝從 TLS session 匯出的 44 bytes 密鑰材料
// 對齊 Rust: ExportedKeyingMaterial struct
type ExportedKeyingMaterial struct {
	// Raw — 完整 44 bytes EKM 原始資料
	Raw []byte
	// ChaChaKey — 前 32 bytes: ChaCha20-Poly1305 加密金鑰
	ChaChaKey []byte
	// ChaChaNonce — 後 12 bytes: ChaCha20-Poly1305 nonce
	ChaChaNonce []byte
	// IsRealEKM — true = 真實 TLS EKM，false = HMAC fallback stub
	IsRealEKM bool
}

// NewExportedKeyingMaterial 從原始 EKM bytes 解構為 key + nonce
func NewExportedKeyingMaterial(raw []byte, isReal bool) (*ExportedKeyingMaterial, error) {
	if len(raw) != TLSExporterLength {
		return nil, fmt.Errorf("EKM 必須為 %d bytes，實際為 %d", TLSExporterLength, len(raw))
	}
	ekm := &ExportedKeyingMaterial{
		Raw:         make([]byte, TLSExporterLength),
		ChaChaKey:   make([]byte, 32),
		ChaChaNonce: make([]byte, 12),
		IsRealEKM:   isReal,
	}
	copy(ekm.Raw, raw)
	copy(ekm.ChaChaKey, raw[0:32])
	copy(ekm.ChaChaNonce, raw[32:44])
	return ekm, nil
}

// =============================================================================
// TLS Exporter Provider 介面 (RFC 5705 §2 / RFC 9266 §3)
// =============================================================================

// TLSExporterProvider — TLS Exporter 提供者介面
// 封裝 TLS session 的 EKM 匯出能力，允許不同的 TLS 後端實作
// 對齊 Rust: TlsExporterProvider trait
type TLSExporterProvider interface {
	// ExportKeyingMaterial 從 TLS session 匯出 Keying Material
	// label: RFC 5705 exporter label
	// context: 應用層上下文 (Session UUID)
	// length: 要匯出的位元組數
	ExportKeyingMaterial(label string, context []byte, length int) ([]byte, error)
}

// =============================================================================
// 真實 TLS Exporter (使用 crypto/tls ConnectionState)
// =============================================================================

// GoTLSExporterProvider — 使用 Go 標準庫 crypto/tls 的真實 TLS Exporter
//
// 透過 tls.ConnectionState.ExportKeyingMaterial() 取得 EKM。
// 此方法在 Go 1.20+ 中可用，支援 TLS 1.3 (RFC 8446 §7.5)
// 以及 TLS 1.2 (RFC 5705)。
//
// # 使用方式
// 在 gRPC interceptor 中從 peer.Peer 取得 *tls.Conn，
// 再取得 ConnectionState 後建立此提供者。
//
// 參考:
//   - Go crypto/tls: ConnectionState.ExportKeyingMaterial
//   - RFC 5705 §2: Exporter Definition
//   - RFC 9266 §3: tls-exporter Channel Binding Type
type GoTLSExporterProvider struct {
	connState tls.ConnectionState
}

// NewGoTLSExporterProvider 從 tls.ConnectionState 建立真實 TLS Exporter 提供者
//
// # 參數
//   - connState: 已完成 TLS handshake 的連線狀態
//     可從 tls.Conn.ConnectionState() 取得
func NewGoTLSExporterProvider(connState tls.ConnectionState) *GoTLSExporterProvider {
	return &GoTLSExporterProvider{connState: connState}
}

// ExportKeyingMaterial 實作 TLSExporterProvider 介面
// 委派至 crypto/tls ConnectionState.ExportKeyingMaterial()
//
// RFC 5705 §2: PRF(master_secret, label, context_value, length) → EKM
// Go 實作自動處理 TLS 1.2/1.3 差異
func (p *GoTLSExporterProvider) ExportKeyingMaterial(label string, context []byte, length int) ([]byte, error) {
	// crypto/tls.ConnectionState.ExportKeyingMaterial(label, context, length)
	// 自 Go 1.20 起支援，底層呼叫 RFC 5705 定義的 PRF
	ekm, err := p.connState.ExportKeyingMaterial(label, context, length)
	if err != nil {
		return nil, fmt.Errorf("crypto/tls ExportKeyingMaterial 失敗: %w", err)
	}
	return ekm, nil
}

// =============================================================================
// HMAC Fallback Provider — 降級方案
// =============================================================================

// HmacFallbackProvider — 當無法取得 TLS 連線時的 HMAC 降級方案
// 使用 HMAC-SHA256(session_uuid, label) 推導密鑰材料
// 對齊 Rust: HmacFallbackProvider
//
// ⚠️ 此模式不具備真正的 TLS 通道綁定安全性，
// 僅作為 TLS 未啟用時的降級方案
type HmacFallbackProvider struct {
	sessionUUID [SessionUUIDLength]byte
}

// NewHmacFallbackProvider 建立 HMAC fallback 提供者
func NewHmacFallbackProvider(sessionUUID [SessionUUIDLength]byte) *HmacFallbackProvider {
	return &HmacFallbackProvider{sessionUUID: sessionUUID}
}

// ExportKeyingMaterial 實作 TLSExporterProvider 介面 (HMAC 模擬)
// 與 Rust derive_ekm_stub / HmacFallbackProvider 行為完全一致
func (p *HmacFallbackProvider) ExportKeyingMaterial(label string, _ []byte, length int) ([]byte, error) {
	// 步驟 1: HMAC-SHA256(session_uuid, label) → 32 bytes key
	mac := hmac.New(sha256.New, p.sessionUUID[:])
	mac.Write([]byte(label))
	keyMaterial := mac.Sum(nil) // 32 bytes

	// 步驟 2: HMAC-SHA256(key_material, "nonce-derivation") → 取前 12 bytes 作為 nonce
	nonceMac := hmac.New(sha256.New, keyMaterial)
	nonceMac.Write([]byte("nonce-derivation"))
	nonceSource := nonceMac.Sum(nil) // 32 bytes

	// 組合: key (32) + nonce (12) = 44 bytes
	result := make([]byte, 0, length)
	if length <= 32 {
		result = append(result, keyMaterial[:length]...)
	} else {
		result = append(result, keyMaterial[:32]...)
		remaining := length - 32
		if remaining > len(nonceSource) {
			remaining = len(nonceSource)
		}
		result = append(result, nonceSource[:remaining]...)
	}

	return result, nil
}

// =============================================================================
// EKM 推導 — 統一入口
// =============================================================================

// DeriveEKM 從 TLSExporterProvider 推導 EKM
// 此為 v1.4 推薦的入口，對齊 Rust: derive_ekm()
//
// # 參數
//   - provider: 實作 TLSExporterProvider 的物件
//   - sessionUUID: 16-byte session 標識符，作為 EKM context
//
// # 回傳
//   - 44 bytes 的 ExportedKeyingMaterial (32 key + 12 nonce)
//
// RFC 參考:
//   - RFC 5705 §2: label + context → EKM
//   - RFC 9266 §3: tls-exporter channel binding
func DeriveEKM(provider TLSExporterProvider, sessionUUID [SessionUUIDLength]byte) *ExportedKeyingMaterial {
	raw, err := provider.ExportKeyingMaterial(
		TLSExporterLabel,
		sessionUUID[:],
		TLSExporterLength,
	)
	if err != nil {
		// EKM 匯出失敗 → 降級為 HMAC fallback
		log.Printf("[WARN] TLS EKM 匯出失敗 (%v), 降級為 HMAC fallback", err)
		return DeriveEKMFallback(sessionUUID)
	}

	_, isHmac := provider.(*HmacFallbackProvider)
	isReal := !isHmac

	ekm, ekmErr := NewExportedKeyingMaterial(raw, isReal)
	if ekmErr != nil {
		log.Printf("[WARN] EKM 建構失敗 (%v), 降級為 HMAC fallback", ekmErr)
		return DeriveEKMFallback(sessionUUID)
	}

	if isReal {
		log.Printf("[INFO] TLS EKM derived (real TLS exporter), label=%s", TLSExporterLabel)
	} else {
		log.Printf("[INFO] TLS EKM derived (HMAC fallback), label=%s", TLSExporterLabel)
	}

	return ekm
}

// DeriveEKMFallback — HMAC Fallback EKM 推導
// 對齊 Rust: derive_ekm_fallback()
// 當無法取得 TLS 連線時使用
func DeriveEKMFallback(sessionUUID [SessionUUIDLength]byte) *ExportedKeyingMaterial {
	provider := NewHmacFallbackProvider(sessionUUID)
	raw, _ := provider.ExportKeyingMaterial(
		TLSExporterLabel,
		sessionUUID[:],
		TLSExporterLength,
	)

	ekm, err := NewExportedKeyingMaterial(raw, false)
	if err != nil {
		// 不應發生，但防呆處理
		log.Printf("[ERROR] HMAC fallback EKM 建構失敗: %v", err)
		return &ExportedKeyingMaterial{
			Raw:         raw,
			ChaChaKey:   raw[:32],
			ChaChaNonce: raw[32:44],
			IsRealEKM:   false,
		}
	}

	log.Printf("[INFO] TLS EKM fallback derived (HMAC-SHA256), label=%s", TLSExporterLabel)
	return ekm
}

// DeriveEKMFromTLSConn — 從 tls.ConnectionState 直接推導真實 EKM
// 便利函數，直接包裝 NewGoTLSExporterProvider + DeriveEKM
//
// # 使用範例
//
//	// 在 gRPC interceptor 中
//	p, _ := peer.FromContext(ctx)
//	if tlsInfo, ok := p.AuthInfo.(credentials.TLSInfo); ok {
//	    ekm := defense.DeriveEKMFromTLSConn(tlsInfo.State, sessionUUID)
//	}
func DeriveEKMFromTLSConn(connState tls.ConnectionState, sessionUUID [SessionUUIDLength]byte) *ExportedKeyingMaterial {
	provider := NewGoTLSExporterProvider(connState)
	return DeriveEKM(provider, sessionUUID)
}

// =============================================================================
// EKM HMAC 驗證
// =============================================================================

// VerifyEKMHmac 驗證 Client 提供的 EKM HMAC 通道綁定
// 對齊 Rust: verify_ekm_hmac()
//
// Client 以 HMAC-SHA256(ekm.Raw, decryptedPlaintext) 計算 clientHmac，
// Daemon 端以相同方式驗證 (constant-time)
//
// # 回傳
//   - true 表示通道綁定一致，Client 確實在同一 TLS session 中完成挑戰
func VerifyEKMHmac(ekm *ExportedKeyingMaterial, decryptedPlaintext, clientHmac []byte) bool {
	mac := hmac.New(sha256.New, ekm.Raw)
	mac.Write(decryptedPlaintext)
	expectedHmac := mac.Sum(nil)

	// constant-time 比對 — 對齊 Rust hmac crate 的 verify_slice
	result := subtle.ConstantTimeCompare(expectedHmac, clientHmac) == 1

	if result {
		log.Printf("[INFO] TLS EKM HMAC 通道綁定驗證成功")
	} else {
		log.Printf("[WARN] TLS EKM HMAC 通道綁定驗證失敗")
	}

	return result
}
