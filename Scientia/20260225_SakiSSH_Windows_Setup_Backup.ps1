# ============================================================
# Saki SSH Windows 環境部署腳本 (Nushell & Registry & Firewall)
# ============================================================

Write-Host "=== Saki Studio SakiSSH 部署與優化 ===" -ForegroundColor Cyan

# 1. 登錄器優化 (抑制 UAC)
Write-Host "[1/3] 配置登錄器以抑制 UAC 彈窗..." -ForegroundColor Yellow
$registryPath = "HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\Policies\System"

Set-ItemProperty -Path $registryPath -Name "ConsentPromptBehaviorAdmin" -Value 0
Set-ItemProperty -Path $registryPath -Name "PromptOnSecureDesktop" -Value 0
Set-ItemProperty -Path $registryPath -Name "LocalAccountTokenFilterPolicy" -Value 1 -ErrorAction SilentlyContinue

Write-Host "  ✅ 登錄器配置完成 (需重啟生效)" -ForegroundColor Green

# 2. 防火牆開放 (Port 19284)
Write-Host "[2/3] 開放防火牆 19284 埠 (SakiSSH)..." -ForegroundColor Yellow
if (Get-NetFirewallRule -DisplayName "SakiSSH" -ErrorAction SilentlyContinue) {
    Remove-NetFirewallRule -DisplayName "SakiSSH"
}
New-NetFirewallRule -DisplayName "SakiSSH" -Direction Inbound -LocalPort 19284 -Protocol TCP -Action Allow
Write-Host "  ✅ 防火牆規則已建立" -ForegroundColor Green

# 3. 部署 sakisshd.exe
Write-Host "[3/3] 確認 Nushell 路徑..." -ForegroundColor Yellow
$nuPath = "C:\Program Files
uin
u.exe"
if (Test-Path $nuPath) {
    Write-Host "  ✅ Nushell 已就緒: $nuPath" -ForegroundColor Green
} else {
    Write-Host "  ⚠️ 警告: 未找到 Nushell，SakiSSH 將無法正常工作" -ForegroundColor Red
}

Write-Host ""
Write-Host "=== 設定完成，建議重啟系統 ===" -ForegroundColor Cyan
