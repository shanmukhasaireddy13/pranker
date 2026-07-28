# Helper script to launch Cloudflare Quick Tunnel for Pranker Server
Write-Host "🌐 Starting Cloudflare Quick Tunnel for Pranker Server (Port 3030)..." -ForegroundColor Cyan
Write-Host "Make sure 'pranker-server.exe' is running first!" -ForegroundColor Yellow

if (Get-Command npx -ErrorAction SilentlyContinue) {
    npx cloudflared tunnel --url http://localhost:3030
} elseif (Get-Command cloudflared -ErrorAction SilentlyContinue) {
    cloudflared tunnel --url http://localhost:3030
} else {
    Write-Host "❌ Neither 'npx' nor 'cloudflared' was found in PATH." -ForegroundColor Red
    Write-Host "Install Node.js or download cloudflared.exe from https://github.com/cloudflare/cloudflared/releases" -ForegroundColor White
}
