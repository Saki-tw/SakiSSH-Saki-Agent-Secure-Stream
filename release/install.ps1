# SakiAgentSSH Windows Installer
# Run as Administrator

param(
    [string]$InstallDir = "C:\SakiSSH",
    [string]$ServiceUser = "saki",
    [switch]$CreateUser,
    [switch]$SkipFirewall
)

$ErrorActionPreference = "Stop"

Write-Host "=== SakiAgentSSH Windows Installer ===" -ForegroundColor Cyan
Write-Host ""

# --- Step 1: Create install directory ---
Write-Host "[1/5] Creating install directory: $InstallDir" -ForegroundColor Yellow
New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null

# --- Step 2: Copy files ---
Write-Host "[2/5] Copying files..." -ForegroundColor Yellow
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path

$DaemonSrc = Join-Path $ScriptDir "daemon\sakisshd.exe"
if (Test-Path $DaemonSrc) {
    Copy-Item $DaemonSrc -Destination (Join-Path $InstallDir "sakisshd.exe") -Force
    Write-Host "  -> sakisshd.exe copied" -ForegroundColor Green
} else {
    Write-Host "  !! sakisshd.exe not found at $DaemonSrc" -ForegroundColor Red
    exit 1
}

$ConfigSrc = Join-Path $ScriptDir "config.json.example"
$ConfigDst = Join-Path $InstallDir "config.json"
if (-not (Test-Path $ConfigDst)) {
    if (Test-Path $ConfigSrc) {
        Copy-Item $ConfigSrc -Destination $ConfigDst
        Write-Host "  -> config.json created from template" -ForegroundColor Green
    } else {
        # Generate default config
        @{
            bind_address = "0.0.0.0:19284"
            shell = @{ type = "powershell"; path = $null; args = $null }
            acl = @{ allowed_cidrs = @(); ed25519_public_keys = @() }
            file_transfer = @{ allowed_paths = @(); max_chunk_size = 65536 }
        } | ConvertTo-Json -Depth 3 | Set-Content $ConfigDst
        Write-Host "  -> config.json generated with defaults" -ForegroundColor Green
    }
} else {
    Write-Host "  -> config.json already exists, skipping" -ForegroundColor DarkYellow
}

# --- Step 3: Firewall ---
if (-not $SkipFirewall) {
    Write-Host "[3/5] Configuring firewall..." -ForegroundColor Yellow
    $existing = Get-NetFirewallRule -DisplayName "SakiAgentSSH" -ErrorAction SilentlyContinue
    if ($existing) {
        Write-Host "  -> Firewall rule already exists" -ForegroundColor DarkYellow
    } else {
        New-NetFirewallRule -DisplayName "SakiAgentSSH" `
            -Direction Inbound -Action Allow -Protocol TCP -LocalPort 19284 `
            -Description "SakiAgentSSH Daemon (gRPC port)" | Out-Null
        Write-Host "  -> Firewall rule created (TCP 19284 Inbound Allow)" -ForegroundColor Green
    }
} else {
    Write-Host "[3/5] Skipping firewall (--SkipFirewall)" -ForegroundColor DarkYellow
}

# --- Step 4: Create user (optional) ---
if ($CreateUser) {
    Write-Host "[4/5] Creating user: $ServiceUser" -ForegroundColor Yellow
    $userExists = Get-LocalUser -Name $ServiceUser -ErrorAction SilentlyContinue
    if ($userExists) {
        Write-Host "  -> User '$ServiceUser' already exists" -ForegroundColor DarkYellow
    } else {
        $password = Read-Host "Enter password for user '$ServiceUser'" -AsSecureString
        New-LocalUser -Name $ServiceUser -Password $password -FullName "SakiSSH Service Account" `
            -Description "SakiAgentSSH daemon service account" | Out-Null
        Add-LocalGroupMember -Group "Users" -Member $ServiceUser -ErrorAction SilentlyContinue
        Write-Host "  -> User '$ServiceUser' created" -ForegroundColor Green
    }

    # Junction for shared tools
    $JunctionPaths = @(
        @{ Link = "C:\Users\$ServiceUser\.cargo"; Target = "C:\Users\$env:USERNAME\.cargo" },
        @{ Link = "C:\Users\$ServiceUser\.rustup"; Target = "C:\Users\$env:USERNAME\.rustup" }
    )
    foreach ($jp in $JunctionPaths) {
        if ((Test-Path $jp.Target) -and -not (Test-Path $jp.Link)) {
            New-Item -ItemType Junction -Path $jp.Link -Target $jp.Target | Out-Null
            Write-Host "  -> Junction: $($jp.Link) -> $($jp.Target)" -ForegroundColor Green
        }
    }
} else {
    Write-Host "[4/5] Skipping user creation (use -CreateUser to enable)" -ForegroundColor DarkYellow
}

# --- Step 5: Summary ---
Write-Host ""
Write-Host "[5/5] Installation complete!" -ForegroundColor Green
Write-Host ""
Write-Host "  Install dir:  $InstallDir" -ForegroundColor Cyan
Write-Host "  Config:       $(Join-Path $InstallDir 'config.json')" -ForegroundColor Cyan
Write-Host "  Daemon:       $(Join-Path $InstallDir 'sakisshd.exe')" -ForegroundColor Cyan
Write-Host ""
Write-Host "  Start daemon:" -ForegroundColor White
Write-Host "    & '$InstallDir\sakisshd.exe'" -ForegroundColor Gray
Write-Host ""
Write-Host "  Start as background process:" -ForegroundColor White
Write-Host "    Start-Process -FilePath '$InstallDir\sakisshd.exe' -WorkingDirectory '$InstallDir' -WindowStyle Hidden" -ForegroundColor Gray
Write-Host ""
Write-Host "  Verify from another machine:" -ForegroundColor White
Write-Host "    sakissh --addr http://<this-ip>:19284 ping" -ForegroundColor Gray
Write-Host ""
