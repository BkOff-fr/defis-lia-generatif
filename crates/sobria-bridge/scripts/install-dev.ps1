# Sobr.ia bridge — installation dev du manifest natif sur Windows (C27.5).
#
# Usage : .\install-dev.ps1 -ExtensionId <ID>
#
# Construit sobria-bridge en release, dépose le manifest au bon endroit
# (registre HKCU + JSON fichier), patche {{BRIDGE_PATH}} + {{EXTENSION_ID}}.

param(
  [Parameter(Mandatory = $true)]
  [string]$ExtensionId
)

$ErrorActionPreference = 'Stop'

$RepoRoot = (Resolve-Path "$PSScriptRoot\..\..\..").Path
$BridgeBin = Join-Path $RepoRoot "target\release\sobria-bridge.exe"

Write-Host "[install-dev] cargo build --release -p sobria-bridge"
cargo build --release -p sobria-bridge --manifest-path "$RepoRoot\Cargo.toml"

if (-not (Test-Path $BridgeBin)) {
  Write-Error "Binaire introuvable : $BridgeBin"
  exit 1
}

$ManifestTemplate = Join-Path $RepoRoot "crates\sobria-bridge\manifest.template.json"
$AppData = $env:APPDATA

# Chrome (HKCU registry + JSON file in any path).
$ChromeJsonDir = Join-Path $AppData "Google\Chrome\NativeMessagingHosts"
New-Item -ItemType Directory -Force -Path $ChromeJsonDir | Out-Null
$ChromeJsonPath = Join-Path $ChromeJsonDir "com.sobria.bridge.json"

# Firefox.
$FirefoxJsonDir = Join-Path $AppData "Mozilla\NativeMessagingHosts"
New-Item -ItemType Directory -Force -Path $FirefoxJsonDir | Out-Null
$FirefoxJsonPath = Join-Path $FirefoxJsonDir "com.sobria.bridge.json"

# Génère le manifest patché.
$Manifest = Get-Content $ManifestTemplate -Raw
$EscapedPath = ($BridgeBin -replace '\\','\\')
$Patched = $Manifest -replace '{{BRIDGE_PATH}}', $EscapedPath `
                     -replace '{{EXTENSION_ID}}', $ExtensionId

$Patched | Set-Content -Path $ChromeJsonPath -Encoding UTF8
$Patched | Set-Content -Path $FirefoxJsonPath -Encoding UTF8
Write-Host "[install-dev] ✓ $ChromeJsonPath"
Write-Host "[install-dev] ✓ $FirefoxJsonPath"

# Registre HKCU pour Chrome.
$ChromeRegPath = "HKCU:\Software\Google\Chrome\NativeMessagingHosts\com.sobria.bridge"
New-Item -Path $ChromeRegPath -Force | Out-Null
Set-ItemProperty -Path $ChromeRegPath -Name '(Default)' -Value $ChromeJsonPath
Write-Host "[install-dev] ✓ HKCU registry $ChromeRegPath"

Write-Host "[install-dev] Manifest natif déployé. Recharge l'extension pour tester."
