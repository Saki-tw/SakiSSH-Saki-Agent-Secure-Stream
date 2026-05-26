package server

import (
	"crypto/ed25519"
	"crypto/rand"
	"crypto/sha256"
	"crypto/x509"
	"encoding/hex"
	"encoding/json"
	"encoding/pem"
	"fmt"
	"os"
	"path/filepath"
	"sync"
	"time"
)

type AuditEvent struct {
	Type       string   `json:"type"`
	SessionID  string   `json:"session_id"`
	AgentName  string   `json:"agent_name"`
	Command    string   `json:"command"`
	Args       []string `json:"args"`
	Cwd        string   `json:"cwd"`
	Allowed    bool     `json:"allowed"`
	DenyReason string   `json:"deny_reason,omitempty"`
}

type AuditRecord struct {
	Timestamp string `json:"timestamp"`
	Type       string   `json:"type"`
	SessionID  string   `json:"session_id"`
	AgentName  string   `json:"agent_name"`
	Command    string   `json:"command"`
	Args       []string `json:"args"`
	Cwd        string   `json:"cwd"`
	Allowed    bool     `json:"allowed"`
	DenyReason string   `json:"deny_reason,omitempty"`
	ChainHash string `json:"chain_hash"`
	Signature string `json:"signature"`
}

type AuditLogger struct {
	mu           sync.Mutex
	privateKey   ed25519.PrivateKey
	publicKeyHex string
	logFile      *os.File
	previousHash string
}

var globalAuditLogger *AuditLogger

func InitAuditLogger() error {
	homeDir, err := os.UserHomeDir()
	if err != nil {
		return err
	}
	configDir := filepath.Join(homeDir, ".config", "sass")
	if err := os.MkdirAll(configDir, 0700); err != nil {
		return err
	}

	keyPath := filepath.Join(configDir, "audit_key.pem")
	pubPath := filepath.Join(configDir, "audit_pub.pem")

	var privKey ed25519.PrivateKey
	var pubKey ed25519.PublicKey

	if _, err := os.Stat(keyPath); os.IsNotExist(err) {
		pubKey, privKey, err = ed25519.GenerateKey(rand.Reader)
		if err != nil {
			return err
		}
		privBytes, err := x509.MarshalPKCS8PrivateKey(privKey)
		if err != nil {
			return err
		}
		pemBlock := &pem.Block{
			Type:  "PRIVATE KEY",
			Bytes: privBytes,
		}
		if err := os.WriteFile(keyPath, pem.EncodeToMemory(pemBlock), 0600); err != nil {
			return err
		}
	} else {
		pemData, err := os.ReadFile(keyPath)
		if err != nil {
			return err
		}
		block, _ := pem.Decode(pemData)
		if block == nil {
			return fmt.Errorf("failed to decode PEM block")
		}
		key, err := x509.ParsePKCS8PrivateKey(block.Bytes)
		if err != nil {
			return err
		}
		privKey = key.(ed25519.PrivateKey)
		pubKey = privKey.Public().(ed25519.PublicKey)
	}

	pubBytes, err := x509.MarshalPKIXPublicKey(pubKey)
	if err != nil {
		return err
	}
	pubPemBlock := &pem.Block{
		Type:  "PUBLIC KEY",
		Bytes: pubBytes,
	}
	pubPemStr := pem.EncodeToMemory(pubPemBlock)
	os.WriteFile(pubPath, pubPemStr, 0644)

	logPath := filepath.Join(configDir, "audit.jsonl")
	file, err := os.OpenFile(logPath, os.O_APPEND|os.O_CREATE|os.O_WRONLY, 0644)
	if err != nil {
		return err
	}

	globalAuditLogger = &AuditLogger{
		privateKey:   privKey,
		publicKeyHex: string(pubPemStr),
		logFile:      file,
		previousHash: "SASS_GENESIS_BLOCK",
	}
	return nil
}

func LogCommandExecute(sessionID, agentName, command string, args []string, cwd string, allowed bool, denyReason string) {
	if globalAuditLogger == nil {
		return
	}
	globalAuditLogger.mu.Lock()
	defer globalAuditLogger.mu.Unlock()

	timestamp := time.Now().UTC().Format(time.RFC3339)
	event := AuditEvent{
		Type:       "CommandExecute",
		SessionID:  sessionID,
		AgentName:  agentName,
		Command:    command,
		Args:       args,
		Cwd:        cwd,
		Allowed:    allowed,
		DenyReason: denyReason,
	}

	eventJSON, _ := json.Marshal(event)

	hasher := sha256.New()
	hasher.Write([]byte(globalAuditLogger.previousHash))
	hasher.Write(eventJSON)
	hasher.Write([]byte(timestamp))
	currentHashBytes := hasher.Sum(nil)
	currentHash := hex.EncodeToString(currentHashBytes)

	sig := ed25519.Sign(globalAuditLogger.privateKey, currentHashBytes)
	sigHex := hex.EncodeToString(sig)

	record := AuditRecord{
		Timestamp:  timestamp,
		Type:       event.Type,
		SessionID:  event.SessionID,
		AgentName:  event.AgentName,
		Command:    event.Command,
		Args:       event.Args,
		Cwd:        event.Cwd,
		Allowed:    event.Allowed,
		DenyReason: event.DenyReason,
		ChainHash:  currentHash,
		Signature:  sigHex,
	}

	recordJSON, _ := json.Marshal(record)
	recordJSON = append(recordJSON, '\n')
	globalAuditLogger.logFile.Write(recordJSON)

	globalAuditLogger.previousHash = currentHash
}

func GetAuditPublicKeyPEM() string {
	if globalAuditLogger != nil {
		return globalAuditLogger.publicKeyHex
	}
	return ""
}
