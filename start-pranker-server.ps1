# ============================================================
# 🎭 Pranker - One-Click Server Startup (Cloudflare Tunnel)
# ============================================================
# Requirements:
#   1. cloudflared.exe installed (https://developers.cloudflare.com/cloudflare-one/connections/connect-networks/get-started/create-local-tunnel/)
#   2. Run ONCE: cloudflared tunnel login
#   3. Run ONCE: cloudflared tunnel create pranker-server
#   4. In Cloudflare DNS: CNAME  prank  ->  <tunnel-id>.cfargotunnel.com
#   5. Run ONCE: cloudflared tunnel route dns pranker-server prank.steamhub.qzz.io
#
# After setup, just run this script every time to start pranking!
# ============================================================

$ErrorActionPreference = "Stop"
$WorkspaceDir = $PSScriptRoot

Write-Host ""
Write-Host "============================================================" -ForegroundColor Cyan
Write-Host "  🎭  PRANKER CONTROL SERVER  +  CLOUDFLARE TUNNEL" -ForegroundColor Cyan
Write-Host "      prank.steamhub.qzz.io" -ForegroundColor Yellow
Write-Host "============================================================" -ForegroundColor Cyan
Write-Host ""

# ── 1. Check cloudflared is installed ──────────────────────
if (-not (Get-Command cloudflared -ErrorAction SilentlyContinue)) {
    Write-Host "❌ cloudflared not found in PATH!" -ForegroundColor Red
    Write-Host ""
    Write-Host "📥 Install it by running:" -ForegroundColor Yellow
    Write-Host "   winget install Cloudflare.cloudflared" -ForegroundColor White
    Write-Host "   -- OR --" -ForegroundColor Gray
    Write-Host "   Download from https://developers.cloudflare.com/cloudflare-one/connections/connect-networks/downloads/" -ForegroundColor White
    Write-Host ""
    Read-Host "Press ENTER to exit"
    exit 1
}

# ── 2. Kill any old server / tunnel processes ──────────────
Write-Host "🔪 Stopping any old Pranker processes..." -ForegroundColor Gray
Get-Process "pranker-server"  -ErrorAction SilentlyContinue | Stop-Process -Force
Get-Process "cloudflared"     -ErrorAction SilentlyContinue | Stop-Process -Force
Start-Sleep -Seconds 1

# ── 3. Build release if binary doesn't exist ──────────────
$ServerExe = Join-Path $WorkspaceDir "target\release\pranker-server.exe"
if (-not (Test-Path $ServerExe)) {
    Write-Host "⚙️  Building release binaries (first run only)..." -ForegroundColor Yellow
    Push-Location $WorkspaceDir
    cargo build --release --workspace
    Pop-Location
}

# ── 4. Launch Pranker Server ───────────────────────────────
Write-Host "🚀 Launching Pranker Server on 127.0.0.1:3030..." -ForegroundColor Green
Start-Process -FilePath $ServerExe -WorkingDirectory $WorkspaceDir -WindowStyle Minimized
Start-Sleep -Seconds 2

# ── 5. Launch Cloudflare Tunnel ────────────────────────────
$ConfigFile = Join-Path $WorkspaceDir "cloudflared-config.yml"
Write-Host "🌐 Launching Cloudflare Tunnel → wss://prank.steamhub.qzz.io ..." -ForegroundColor Magenta

Start-Process powershell -ArgumentList "-NoExit -Command cloudflared tunnel --config `"$ConfigFile`" run pranker-server" -WindowStyle Normal
Start-Sleep -Seconds 3

# ── 6. Open Dashboard ──────────────────────────────────────
Write-Host "💻 Opening Control Dashboard..." -ForegroundColor Green
Start-Process "http://localhost:3030"

Write-Host ""
Write-Host "============================================================" -ForegroundColor Cyan
Write-Host "  ✅  EVERYTHING IS READY!" -ForegroundColor Green
Write-Host ""
Write-Host "  🌍 Public WebSocket : wss://prank.steamhub.qzz.io/ws" -ForegroundColor Yellow
Write-Host "  🖥️  Local Dashboard  : http://localhost:3030" -ForegroundColor Yellow
Write-Host "  📦 Client EXE       : target\release\pranker-client.exe" -ForegroundColor Yellow
Write-Host ""
Write-Host "  👉 Copy 'pranker-client.exe' to your friend's laptop!" -ForegroundColor Green
Write-Host "============================================================" -ForegroundColor Cyan
Write-Host ""
