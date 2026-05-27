// =============================================================================
// Package defense — localhost_defense.go
// SASS v1.4 LocalHost Agent 防禦機制
//
// 對應 Rust: localhost_defense.rs
// RFC 參考: draft-sakistudio-sass-00 §7.1 (localhost-spoofing)
//
// 當偵測到來自 localhost 的未認證請求時，透過偽造儲存空間與記憶體資訊，
// 誤導可能潛入的惡意 Agent，並保護本機的高權限憑證。
//
// v1.4 升級: 加入動態微隨機化 (Live OS Simulation) 與動態 XOR 混淆掩碼
// Phase 4 升級: 32-byte session key + Base64 編碼封裝
//
// Copyright (c) 2026 Saki Studio. All rights reserved.
// =============================================================================

package defense

import (
	"crypto/rand"
	"encoding/base64"
	"encoding/hex"
	"fmt"
	mrand "math/rand"
	"strings"
)

// XorObfuscateOutput 使用 32-byte session key 對資料進行 repeating-key XOR 混淆，
// 並以 Base64 編碼封裝輸出。
// 對齊 Rust: xor_obfuscate_output()
//
// # 設計原理
//   - 使用完整 32-byte key 進行 repeating-key XOR，相較單位元組 XOR 大幅提高破解難度
//   - 輸出經 Base64 編碼，確保可安全傳輸於文字協議中
func XorObfuscateOutput(data []byte, sessionKey [32]byte) []byte {
	xored := make([]byte, len(data))
	for i, b := range data {
		xored[i] = b ^ sessionKey[i%32]
	}
	return []byte(base64.StdEncoding.EncodeToString(xored))
}

// DeobfuscateOutput 將 Base64 編碼的混淆資料還原為原始位元組。
// 對齊 Rust: deobfuscate_output()
//
// 為 XorObfuscateOutput 的逆運算
func DeobfuscateOutput(b64Data string, sessionKey [32]byte) ([]byte, error) {
	xored, err := base64.StdEncoding.DecodeString(b64Data)
	if err != nil {
		return nil, fmt.Errorf("Base64 解碼失敗: %w", err)
	}
	original := make([]byte, len(xored))
	for i, b := range xored {
		original[i] = b ^ sessionKey[i%32]
	}
	return original, nil
}

// HandleSpoofing 攔截未認證的 Localhost 請求，若符合特定探測指令則回傳偽造資料
// 對齊 Rust: handle_spoofing()
//
// 偵測的探測指令類型:
//  1. df / statvfs → 儲存空間偽造（微幅隨機化）
//  2. meminfo / hw.memsize / free → 記憶體值區偽造
//  3. .aws/credentials / env / export → 憑證/環境變數 XOR 動態混淆
//
// # 回傳
//   - nil: 非探測指令，不攔截
//   - []byte: 偽造的輸出資料
func HandleSpoofing(command string, args []string) []byte {
	fullCommand := command + " " + strings.Join(args, " ")

	// 1. 儲存空間偽造 (Storage Spoofing) - 微幅動態隨機化
	if strings.Contains(fullCommand, "df") || strings.Contains(fullCommand, "statvfs") {
		freeBlocks := mrand.Intn(512)
		usedBlocks := 1953595392 - freeBlocks
		fakeDf := fmt.Sprintf(
			"Filesystem   512-blocks      Used Available Capacity iused      ifree %%iused  Mounted on\n"+
				"/dev/disk3s1s1 1953595392 %d         %d   100%% 1056557 9766920403    0%%   /\n"+
				"devfs                 691       691         0   100%%    1200          0  100%%   /dev\n",
			usedBlocks, freeBlocks)
		return []byte(fakeDf)
	}

	// 2. 記憶體值區偽造 (Memory Region Spoofing) - 微幅動態隨機化
	if strings.Contains(fullCommand, "meminfo") ||
		strings.Contains(fullCommand, "hw.memsize") ||
		strings.Contains(fullCommand, "free") {

		freeKB := 8192 + mrand.Intn(8192) // 8MB ~ 16MB
		usedKB := 262144 - freeKB

		var fakeMem string
		if strings.Contains(fullCommand, "hw.memsize") {
			fakeMem = "hw.memsize: 268435456\n" // 256 MB static hardware cap
		} else if strings.Contains(fullCommand, "free") {
			fakeMem = fmt.Sprintf(
				"              total        used        free      shared  buff/cache   available\n"+
					"Mem:          262144      %d       %d           0           0       %d\n"+
					"Swap:              0           0           0\n",
				usedKB, freeKB, freeKB)
		} else {
			fakeMem = fmt.Sprintf(
				"MemTotal:         262144 kB\n"+
					"MemFree:           %d kB\n"+
					"MemAvailable:      %d kB\n",
				freeKB, freeKB)
		}
		return []byte(fakeMem)
	}

	// 3. 憑證/環境變數 XOR 動態混淆
	if strings.Contains(fullCommand, ".aws/credentials") ||
		strings.Contains(fullCommand, "env") ||
		strings.Contains(fullCommand, "export") {

		rawCreds := make([]byte, 256)
		rand.Read(rawCreds)

		xorKey := byte(1 + mrand.Intn(254))
		for i := range rawCreds {
			rawCreds[i] ^= xorKey
		}

		hexCreds := hex.EncodeToString(rawCreds)
		output := fmt.Sprintf(
			"[default]\n"+
				"aws_access_key_id = AKIA_SASS_XOR_%s\n"+
				"aws_secret_access_key = sass_obfuscated_xor_key_%d_len_%d\n",
			hexCreds[:16], xorKey, len(hexCreds))
		return []byte(output)
	}

	return nil
}
