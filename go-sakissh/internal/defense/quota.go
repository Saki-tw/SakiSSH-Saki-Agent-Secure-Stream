// =============================================================================
// Package defense — quota.go
// SASS v1.4 資源配額與排隊管理器 (Resource Quota & Queuing Manager)
//
// 對應 Rust: quota.rs (ResourceQuotaManager)
// RFC 參考: draft-sakistudio-sass-00 §5.2 (resource-quota)
//
// 防禦 Agent Teams 高併發請求，避免 OS 資源耗盡。
// 加入 Queue 深度上限，防禦記憶體爆破 (OOM Attack)。
//
// Copyright (c) 2026 Saki Studio. All rights reserved.
// =============================================================================

package defense

import (
	"fmt"
	"log"
	"sync"
)

// IdentityQuota — 每個 ED25519 身分的配額狀態
// 對齊 Rust: IdentityQuota struct
type IdentityQuota struct {
	// ActivePTYs — 目前使用中的 PTY 數量
	ActivePTYs int
	// MaxPTYs — 此身分的最大 PTY 配額
	MaxPTYs int
	// WaitQueue — 等待佇列中的請求通道
	WaitQueue []chan struct{}
}

// ResourceQuotaManager — 資源配額管理器 (Thread-safe)
// 對齊 Rust: ResourceQuotaManager struct
//
// 設計原則:
//   - 每個 ED25519 identity 有獨立的 PTY 配額
//   - 超過配額時進入佇列等待
//   - 佇列深度有上限，防禦 DDoS 記憶體耗盡攻擊
//
// RFC 參考: draft-sakistudio-sass-00 §5.2 (resource-quota)
type ResourceQuotaManager struct {
	mu             sync.Mutex
	quotas         map[string]*IdentityQuota
	defaultMaxPTYs int
	// MaxQueueDepth — 防禦 Agent 惡意 spam 導致佇列爆破記憶體的上限
	maxQueueDepth int
}

// NewResourceQuotaManager 建立資源配額管理器
// 對齊 Rust: ResourceQuotaManager::new()
//
// # 參數
//   - defaultMaxPTYs: 每個身分的預設最大 PTY 數
//   - maxQueueDepth: 佇列深度上限（防禦 DDoS）
func NewResourceQuotaManager(defaultMaxPTYs, maxQueueDepth int) *ResourceQuotaManager {
	return &ResourceQuotaManager{
		quotas:         make(map[string]*IdentityQuota),
		defaultMaxPTYs: defaultMaxPTYs,
		maxQueueDepth:  maxQueueDepth,
	}
}

// AcquirePTY 申請執行權限
// 對齊 Rust: ResourceQuotaManager::acquire_pty()
//
// 回傳值:
//   - (nil, nil): 直接放行，無需等待
//   - (chan, nil): 需要等待，監聽回傳的 channel
//   - (nil, error): 佇列已滿，拒絕服務（可能為 DDoS 攻擊）
func (m *ResourceQuotaManager) AcquirePTY(identityPubkey string) (chan struct{}, error) {
	m.mu.Lock()
	defer m.mu.Unlock()

	quota, exists := m.quotas[identityPubkey]
	if !exists {
		quota = &IdentityQuota{
			ActivePTYs: 0,
			MaxPTYs:    m.defaultMaxPTYs,
			WaitQueue:  make([]chan struct{}, 0),
		}
		m.quotas[identityPubkey] = quota
	}

	if quota.ActivePTYs < quota.MaxPTYs {
		// 直接放行
		quota.ActivePTYs++
		return nil, nil
	}

	// 檢查佇列深度上限 — 防禦 DDoS
	if len(quota.WaitQueue) >= m.maxQueueDepth {
		return nil, fmt.Errorf(
			"Quota Exceeded and Queue is Full. Possible DoS attack detected. "+
				"identity=%s, active=%d, max=%d, queue_depth=%d",
			identityPubkey, quota.ActivePTYs, quota.MaxPTYs, len(quota.WaitQueue))
	}

	// 加入等待佇列
	notify := make(chan struct{}, 1)
	quota.WaitQueue = append(quota.WaitQueue, notify)
	log.Printf("[INFO] Quota: identity=%s queued at position %d", identityPubkey, len(quota.WaitQueue))
	return notify, nil
}

// ReleasePTY 釋放執行權限，並喚醒 Queue 中的下一個請求
// 對齊 Rust: ResourceQuotaManager::release_pty()
func (m *ResourceQuotaManager) ReleasePTY(identityPubkey string) {
	m.mu.Lock()
	defer m.mu.Unlock()

	quota, exists := m.quotas[identityPubkey]
	if !exists {
		return
	}

	if len(quota.WaitQueue) > 0 {
		// 有人在排隊，直接把名額轉讓給他並喚醒
		notify := quota.WaitQueue[0]
		quota.WaitQueue = quota.WaitQueue[1:]
		close(notify) // 喚醒等待者
	} else {
		if quota.ActivePTYs > 0 {
			quota.ActivePTYs--
		}
	}
}

// GetQueuePosition 取得目前排隊的位置
// 對齊 Rust: ResourceQuotaManager::get_queue_position()
func (m *ResourceQuotaManager) GetQueuePosition(identityPubkey string) int {
	m.mu.Lock()
	defer m.mu.Unlock()

	quota, exists := m.quotas[identityPubkey]
	if !exists {
		return 0
	}
	return len(quota.WaitQueue)
}
