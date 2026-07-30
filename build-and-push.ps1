# Automated Build and Deploy Script for Pranker Client
# This script increments the patch version in crates/pranker-client/Cargo.toml, cleans/rebuilds, and pushes to Git.

$tomlPath = "crates/pranker-client/Cargo.toml"
$content = Get-Content $tomlPath -Raw

if ($content -match 'version\s*=\s*"([^"]+)"') {
    $oldVersion = $Matches[1]
    $parts = $oldVersion.Split('.')
    if ($parts.Length -eq 3) {
        $patch = [int]$parts[2] + 1
        $newVersion = "$($parts[0]).$($parts[1]).$patch"
        
        $newContent = $content -replace ('version\s*=\s*"' + [regex]::Escape($oldVersion) + '"'), ('version = "' + $newVersion + '"')
        Set-Content $tomlPath $newContent -NoNewline
        Write-Host "✅ Bumped version from $oldVersion to $newVersion in Cargo.toml"
    } else {
        Write-Error "❌ Invalid version format in Cargo.toml: $oldVersion"
        exit 1
    }
} else {
    Write-Error "❌ Could not find version in Cargo.toml"
    exit 1
}

# Clean old targets to ensure no cached version strings are used
Write-Host "🧹 Cleaning old builds..."
cargo clean

# Build the system-admin release binary
Write-Host "⚙️ Building system-admin release binary..."
cargo build --release --bin system-admin
if ($LASTEXITCODE -ne 0) {
    Write-Error "❌ Cargo build failed!"
    exit 1
}

# Terminate active client instances so the binary file can be overwritten
Write-Host "💀 Terminating any active system-admin.exe processes..."
taskkill /F /IM system-admin.exe 2>$null

# Copy binary to project root
$destPath = "system-admin.exe"
Write-Host "🚚 Copying binary to project root..."
Copy-Item -Path "target/release/system-admin.exe" -Destination $destPath -Force

# Git Commit and Push
Write-Host "🐙 Staging and committing to git..."
git add .
$env:GIT_EDITOR="true"
git commit -m "bump: upgrade system-admin client to v$newVersion"

Write-Host "🔄 Pulling remote modifications..."
git pull --rebase origin main

Write-Host "🚀 Pushing update to origin main..."
git push origin main

Write-Host "🎉 Successfully deployed v$newVersion!"
