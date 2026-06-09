// =============================================================================
// Package defense — tls_exporter_test.go
// SASS Plugin #2: TLS Exporter Binding 單元測試
//
// 對照 C# PluginTests.cs: TlsExporterBindingTests
// Copyright (c) 2026 Saki Studio. All rights reserved.
// =============================================================================

package defense

import (
	"bytes"
	"crypto/hmac"
	"crypto/sha256"
	"testing"
)

// --- 常量驗證 ---

func TestTLSExporterLabel_MatchesRfc(t *testing.T) {
	t.Parallel()
	expected := "EXPORTER-sakissh-chacha20-v15"
	if TLSExporterLabel != expected {
		t.Errorf("TLSExporterLabel 應為 %q，實際為 %q", expected, TLSExporterLabel)
	}
}

func TestTLSExporterLength_Is44(t *testing.T) {
	t.Parallel()
	if TLSExporterLength != 44 {
		t.Errorf("TLSExporterLength 應為 44，實際為 %d", TLSExporterLength)
	}
}

func TestSessionUUIDLength_Is16(t *testing.T) {
	t.Parallel()
	if SessionUUIDLength != 16 {
		t.Errorf("SessionUUIDLength 應為 16，實際為 %d", SessionUUIDLength)
	}
}

// --- NewExportedKeyingMaterial ---

func TestNewExportedKeyingMaterial_ValidInput(t *testing.T) {
	t.Parallel()
	raw := make([]byte, 44)
	for i := range raw {
		raw[i] = byte(i)
	}

	ekm, err := NewExportedKeyingMaterial(raw, true)
	if err != nil {
		t.Fatalf("不應回傳錯誤: %v", err)
	}
	if len(ekm.Raw) != 44 {
		t.Errorf("Raw 長度應為 44，實際為 %d", len(ekm.Raw))
	}
	if len(ekm.ChaChaKey) != 32 {
		t.Errorf("ChaChaKey 長度應為 32，實際為 %d", len(ekm.ChaChaKey))
	}
	if len(ekm.ChaChaNonce) != 12 {
		t.Errorf("ChaChaNonce 長度應為 12，實際為 %d", len(ekm.ChaChaNonce))
	}
	if !ekm.IsRealEKM {
		t.Error("IsRealEKM 應為 true")
	}
}

func TestNewExportedKeyingMaterial_InvalidLength_ReturnsError(t *testing.T) {
	t.Parallel()
	shortRaw := make([]byte, 8)
	_, err := NewExportedKeyingMaterial(shortRaw, false)
	if err == nil {
		t.Error("長度不為 44 的輸入應回傳錯誤")
	}
}

func TestNewExportedKeyingMaterial_KeyAndNonceMatchRaw(t *testing.T) {
	t.Parallel()
	raw := make([]byte, 44)
	for i := range raw {
		raw[i] = byte(i + 10)
	}

	ekm, err := NewExportedKeyingMaterial(raw, false)
	if err != nil {
		t.Fatalf("不應回傳錯誤: %v", err)
	}

	// ChaChaKey 應為 raw[0:32]
	if !bytes.Equal(ekm.ChaChaKey, raw[0:32]) {
		t.Error("ChaChaKey 應等於 raw[0:32]")
	}
	// ChaChaNonce 應為 raw[32:44]
	if !bytes.Equal(ekm.ChaChaNonce, raw[32:44]) {
		t.Error("ChaChaNonce 應等於 raw[32:44]")
	}
}

// --- DeriveEKMFallback (HMAC) ---

func TestDeriveEKMFallback_Returns44Bytes(t *testing.T) {
	t.Parallel()
	var sessionUUID [16]byte
	for i := range sessionUUID {
		sessionUUID[i] = byte(i * 3)
	}

	ekm := DeriveEKMFallback(sessionUUID)

	if len(ekm.Raw) != 44 {
		t.Errorf("EKM Raw 長度應為 44，實際為 %d", len(ekm.Raw))
	}
	if len(ekm.ChaChaKey) != 32 {
		t.Errorf("ChaChaKey 長度應為 32，實際為 %d", len(ekm.ChaChaKey))
	}
	if len(ekm.ChaChaNonce) != 12 {
		t.Errorf("ChaChaNonce 長度應為 12，實際為 %d", len(ekm.ChaChaNonce))
	}
}

func TestDeriveEKMFallback_SameInput_ProducesSameOutput(t *testing.T) {
	t.Parallel()
	var sessionUUID [16]byte
	for i := range sessionUUID {
		sessionUUID[i] = byte(42 + i)
	}

	ekm1 := DeriveEKMFallback(sessionUUID)
	ekm2 := DeriveEKMFallback(sessionUUID)

	if !bytes.Equal(ekm1.Raw, ekm2.Raw) {
		t.Error("相同的 session UUID 應產生相同的 EKM（確定性）")
	}
}

func TestDeriveEKMFallback_DifferentInput_ProducesDifferentOutput(t *testing.T) {
	t.Parallel()
	var uuid1, uuid2 [16]byte
	for i := range uuid1 {
		uuid1[i] = byte(i)
		uuid2[i] = byte(i + 100)
	}

	ekm1 := DeriveEKMFallback(uuid1)
	ekm2 := DeriveEKMFallback(uuid2)

	if bytes.Equal(ekm1.Raw, ekm2.Raw) {
		t.Error("不同的 session UUID 應產生不同的 EKM")
	}
}

func TestDeriveEKMFallback_IsNotRealEKM(t *testing.T) {
	t.Parallel()
	var sessionUUID [16]byte
	ekm := DeriveEKMFallback(sessionUUID)
	if ekm.IsRealEKM {
		t.Error("HMAC fallback EKM 的 IsRealEKM 應為 false")
	}
}

// --- DeriveEKM with HmacFallbackProvider ---

func TestDeriveEKM_WithHmacFallback_Returns44Bytes(t *testing.T) {
	t.Parallel()
	var sessionUUID [16]byte
	for i := range sessionUUID {
		sessionUUID[i] = byte(i)
	}

	provider := NewHmacFallbackProvider(sessionUUID)
	ekm := DeriveEKM(provider, sessionUUID)

	if len(ekm.Raw) != 44 {
		t.Errorf("EKM Raw 長度應為 44，實際為 %d", len(ekm.Raw))
	}
	if ekm.IsRealEKM {
		t.Error("使用 HmacFallbackProvider 時 IsRealEKM 應為 false")
	}
}

// --- HmacFallbackProvider ---

func TestHmacFallbackProvider_ExportKeyingMaterial_Deterministic(t *testing.T) {
	t.Parallel()
	var sessionUUID [16]byte
	for i := range sessionUUID {
		sessionUUID[i] = byte(i + 7)
	}

	provider := NewHmacFallbackProvider(sessionUUID)
	result1, err1 := provider.ExportKeyingMaterial(TLSExporterLabel, sessionUUID[:], TLSExporterLength)
	result2, err2 := provider.ExportKeyingMaterial(TLSExporterLabel, sessionUUID[:], TLSExporterLength)

	if err1 != nil || err2 != nil {
		t.Fatalf("ExportKeyingMaterial 不應回傳錯誤: err1=%v, err2=%v", err1, err2)
	}
	if !bytes.Equal(result1, result2) {
		t.Error("相同輸入應產生相同輸出")
	}
}

func TestHmacFallbackProvider_ExportKeyingMaterial_CorrectLength(t *testing.T) {
	t.Parallel()
	var sessionUUID [16]byte
	provider := NewHmacFallbackProvider(sessionUUID)

	for _, length := range []int{16, 32, 44} {
		result, err := provider.ExportKeyingMaterial(TLSExporterLabel, sessionUUID[:], length)
		if err != nil {
			t.Fatalf("ExportKeyingMaterial (length=%d) 失敗: %v", length, err)
		}
		if len(result) != length {
			t.Errorf("length=%d 時，結果長度應為 %d，實際為 %d", length, length, len(result))
		}
	}
}

// --- VerifyEKMHmac ---

func TestVerifyEKMHmac_ValidHmac_ReturnsTrue(t *testing.T) {
	t.Parallel()
	var sessionUUID [16]byte
	for i := range sessionUUID {
		sessionUUID[i] = byte(42 + i)
	}

	ekm := DeriveEKMFallback(sessionUUID)
	plaintext := []byte("test-plaintext")

	// 計算正確的 HMAC
	mac := hmac.New(sha256.New, ekm.Raw)
	mac.Write(plaintext)
	correctHmac := mac.Sum(nil)

	if !VerifyEKMHmac(ekm, plaintext, correctHmac) {
		t.Error("正確的 HMAC 應驗證成功")
	}
}

func TestVerifyEKMHmac_InvalidHmac_ReturnsFalse(t *testing.T) {
	t.Parallel()
	var sessionUUID [16]byte
	for i := range sessionUUID {
		sessionUUID[i] = byte(42 + i)
	}

	ekm := DeriveEKMFallback(sessionUUID)
	plaintext := []byte("test-plaintext")
	wrongHmac := make([]byte, 32) // 全零的 HMAC

	if VerifyEKMHmac(ekm, plaintext, wrongHmac) {
		t.Error("錯誤的 HMAC 應驗證失敗")
	}
}

func TestVerifyEKMHmac_DifferentPlaintext_ReturnsFalse(t *testing.T) {
	t.Parallel()
	var sessionUUID [16]byte
	ekm := DeriveEKMFallback(sessionUUID)

	plaintext1 := []byte("original-plaintext")
	plaintext2 := []byte("tampered-plaintext")

	mac := hmac.New(sha256.New, ekm.Raw)
	mac.Write(plaintext1)
	hmacForPlaintext1 := mac.Sum(nil)

	// 用 plaintext1 的 HMAC 驗證 plaintext2
	if VerifyEKMHmac(ekm, plaintext2, hmacForPlaintext1) {
		t.Error("不同 plaintext 的 HMAC 應驗證失敗")
	}
}
