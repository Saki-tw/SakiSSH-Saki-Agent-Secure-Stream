package defense

import (
	"crypto/rand"
	"sync"
	"time"

	"golang.org/x/crypto/chacha20poly1305"
)

type ChallengeEntry struct {
	Key       []byte
	Nonce     []byte
	Plaintext []byte
	CreatedAt time.Time
	TTL       time.Duration
}

type ChallengeStore struct {
	mu         sync.RWMutex
	entries    map[string]*ChallengeEntry
	defaultTTL time.Duration
}

func NewChallengeStore(ttlSeconds int) *ChallengeStore {
	store := &ChallengeStore{
		entries:    make(map[string]*ChallengeEntry),
		defaultTTL: time.Duration(ttlSeconds) * time.Second,
	}
	go store.cleanupLoop()
	return store
}

func (s *ChallengeStore) GenerateChallenge() ([]byte, []byte, error) {
	key := make([]byte, 32)
	rand.Read(key)

	nonce := make([]byte, 12)
	rand.Read(nonce)

	plaintext := make([]byte, 64)
	rand.Read(plaintext)

	aead, err := chacha20poly1305.New(key)
	if err != nil {
		return nil, nil, err
	}

	ciphertext := aead.Seal(nil, nonce, plaintext, nil)

	entry := &ChallengeEntry{
		Key:       key,
		Nonce:     nonce,
		Plaintext: plaintext,
		CreatedAt: time.Now(),
		TTL:       s.defaultTTL,
	}

	s.mu.Lock()
	s.entries[string(nonce)] = entry
	s.mu.Unlock()

	return nonce, ciphertext, nil
}

func (s *ChallengeStore) VerifyResponse(nonce []byte, response []byte) bool {
	s.mu.Lock()
	entry, exists := s.entries[string(nonce)]
	if exists {
		delete(s.entries, string(nonce))
	}
	s.mu.Unlock()

	if !exists {
		return false
	}

	if time.Since(entry.CreatedAt) > entry.TTL {
		return false
	}

	if len(response) != len(entry.Plaintext) {
		return false
	}

	for i := range response {
		if response[i] != entry.Plaintext[i] {
			return false
		}
	}
	return true
}

func (s *ChallengeStore) cleanupLoop() {
	ticker := time.NewTicker(60 * time.Second)
	defer ticker.Stop()
	for range ticker.C {
		s.cleanupExpired()
	}
}

func (s *ChallengeStore) cleanupExpired() {
	s.mu.Lock()
	defer s.mu.Unlock()
	now := time.Now()
	for k, v := range s.entries {
		if now.Sub(v.CreatedAt) > v.TTL {
			delete(s.entries, k)
		}
	}
}
