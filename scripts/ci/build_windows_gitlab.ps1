# GitLab.com SaaS Windows runner 的 Windows 便携版构建。
#
# 该脚本由 .gitlab-ci.yml 调用，默认兼容 Windows PowerShell 5.1；
# 它也可以由 PowerShell 7（pwsh）执行。所有工具链都放在临时目录，
# 不污染源码工作区，也不依赖 SaaS runner 的跨 job 持久缓存。

param(
    [Parameter(Mandatory = $true)]
    [string]$Tag
)

$ErrorActionPreference = "Stop"
$ProgressPreference = "SilentlyContinue"
$RepoRoot = Split-Path -Parent (Split-Path -Parent (Split-Path -Parent $PSCommandPath))
Set-Location $RepoRoot

function Invoke-Native {
    param(
        [Parameter(Mandatory = $true)] [string]$Description,
        [Parameter(Mandatory = $true)] [scriptblock]$Command
    )
    Write-Host "== $Description =="
    & $Command
    if ($LASTEXITCODE -ne 0) {
        throw "$Description failed with exit code $LASTEXITCODE"
    }
}

function Resolve-CommandPath {
    param([Parameter(Mandatory = $true)] [string]$Name)
    $command = Get-Command $Name -ErrorAction SilentlyContinue
    if ($command) { return $command.Source }
    return $null
}

$env:PYTHONIOENCODING = "utf-8"
$ToolRoot = Join-Path $env:TEMP "copypolish-gitlab-tools"
$env:CARGO_HOME = Join-Path $ToolRoot "cargo"
$env:RUSTUP_HOME = Join-Path $ToolRoot "rustup"
$NodeRoot = Join-Path $ToolRoot "node"
New-Item -ItemType Directory -Force -Path $ToolRoot, $env:CARGO_HOME, $env:RUSTUP_HOME | Out-Null

Write-Host "PowerShell: $($PSVersionTable.PSVersion)"
Write-Host "Git: $(git --version)"

# ---- Visual Studio / MSVC / Windows SDK 预检 ---------------------------------
$vswhereCandidates = @(
    (Join-Path ${env:ProgramFiles(x86)} "Microsoft Visual Studio\Installer\vswhere.exe"),
    (Resolve-CommandPath "vswhere.exe")
) | Where-Object { $_ -and (Test-Path $_) }
$vswhere = $vswhereCandidates | Select-Object -First 1
if (-not $vswhere) {
    throw "vswhere.exe was not found; Visual Studio Build Tools / Windows SDK cannot be verified."
}

$vsPath = & $vswhere -latest -products * `
    -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 `
    -property installationPath
if (-not $vsPath) {
    throw "No Visual Studio installation with VC.Tools.x86.x64 was found."
}

$vsDevCmd = Join-Path $vsPath "Common7\Tools\VsDevCmd.bat"
if (-not (Test-Path $vsDevCmd)) {
    throw "VsDevCmd.bat was not found: $vsDevCmd"
}

# 将 VsDevCmd 设置的环境变量导入当前 PowerShell 进程。
cmd.exe /s /c "`"$vsDevCmd`" -arch=x64 -host_arch=x64 && set" |
    ForEach-Object {
        if ($_ -match "^([^=]+)=(.*)$") {
            [Environment]::SetEnvironmentVariable($matches[1], $matches[2], "Process")
        }
    }

foreach ($tool in @("cl.exe", "link.exe", "rc.exe", "mt.exe")) {
    if (-not (Resolve-CommandPath $tool)) {
        throw "MSVC/Windows SDK tool is unavailable: $tool"
    }
}

# ---- Node 24.19.0 -------------------------------------------------------------
$nodeVersion = (node --version).Trim()
if ($nodeVersion -ne "v24.19.0") {
    $nodeArchive = Join-Path $env:TEMP "node-v24.19.0-win-x64.zip"
    $nodeExtract = Join-Path $env:TEMP "node-v24.19.0-win-x64"
    Invoke-WebRequest `
        -Uri "https://nodejs.org/dist/v24.19.0/node-v24.19.0-win-x64.zip" `
        -OutFile $nodeArchive
    Expand-Archive -Path $nodeArchive -DestinationPath $env:TEMP -Force
    if (Test-Path $NodeRoot) { Remove-Item -Recurse -Force $NodeRoot }
    Move-Item $nodeExtract $NodeRoot -Force
    $env:PATH = "$NodeRoot;$env:PATH"
}
if ((node --version).Trim() -ne "v24.19.0") {
    throw "Node version mismatch; expected v24.19.0, got $((node --version).Trim())"
}

# ---- Rust 1.98.0 MSVC --------------------------------------------------------
$cargoBin = Join-Path $env:CARGO_HOME "bin"
$rustupInit = Join-Path $env:TEMP "rustup-init.exe"
if (-not (Test-Path (Join-Path $cargoBin "rustc.exe"))) {
    Invoke-WebRequest -Uri "https://win.rustup.rs/x86_64" -OutFile $rustupInit
    & $rustupInit -y --profile minimal
    $rustup = Join-Path $cargoBin "rustup.exe"
    & $rustup toolchain install 1.98.0-x86_64-pc-windows-msvc --profile minimal
    & $rustup default 1.98.0-x86_64-pc-windows-msvc
}
$env:PATH = "$cargoBin;$env:PATH"
if ((rustc --version) -notmatch "1\.98\.0") {
    throw "Rust version mismatch: $(rustc --version)"
}
$rustHost = (rustc -vV | Select-String '^host:' | Select-Object -First 1).Line.Trim()
if ($rustHost -ne "host: x86_64-pc-windows-msvc") {
    throw "Rust host mismatch: $rustHost"
}

# ---- Python / 7-Zip -----------------------------------------------------------
$python = Resolve-CommandPath "python.exe"
if (-not $python) { throw "python.exe was not found" }
$sevenZip = Resolve-CommandPath "7z.exe"
if (-not $sevenZip) { throw "7z.exe was not found" }

Write-Host "Node: $(node --version), npm: $(npm --version)"
Write-Host "Rust: $(rustc --version)"
Write-Host "Python: $(& $python --version 2>&1)"
Write-Host "7-Zip: $sevenZip"

# ---- 版本与构建 ---------------------------------------------------------------
Invoke-Native "Sync version $Tag" { & $python "$RepoRoot\scripts\prepare_release_version.py" $Tag }
Invoke-Native "Verify version $Tag" { & $python "$RepoRoot\scripts\check_version.py" $Tag }

Push-Location (Join-Path $RepoRoot "frontend")
try {
    Invoke-Native "Install frontend dependencies" { npm ci }
    Invoke-Native "Build Tauri Windows exe" { npm run tauri -- build --no-bundle }
}
finally {
    Pop-Location
}

$exePath = Join-Path $RepoRoot "src-tauri\target\release\chinese-copywriting-formatter.exe"
if (-not (Test-Path $exePath)) { throw "Build output was not found: $exePath" }

# ---- 便携版打包 ---------------------------------------------------------------
$dist = Join-Path $RepoRoot "dist\windows"
$staging = Join-Path $env:TEMP "copypolish-windows-staging"
if (Test-Path $staging) { Remove-Item -Recurse -Force $staging }
New-Item -ItemType Directory -Force -Path $staging, $dist | Out-Null
Copy-Item $exePath (Join-Path $staging "CopyPolish.exe")
Get-ChildItem (Split-Path $exePath) -Filter "*.dll" -File -ErrorAction SilentlyContinue |
    Copy-Item -Destination $staging

$archive = Join-Path $dist "CopyPolish-windows-x64.7z"
if (Test-Path $archive) { Remove-Item -Force $archive }
Push-Location $staging
try {
    $files = Get-ChildItem -File | ForEach-Object { $_.Name }
    & $sevenZip a -t7z -mx=9 $archive $files
    if ($LASTEXITCODE -ne 0) { throw "7-Zip failed with exit code $LASTEXITCODE" }
}
finally {
    Pop-Location
}
Copy-Item (Join-Path $staging "CopyPolish.exe") (Join-Path $dist "CopyPolish.exe") -Force

# ---- TUI 独立资产 -------------------------------------------------------------
Invoke-Native "Build TUI release binary" {
    cargo build `
        --manifest-path "$RepoRoot\src-tauri\Cargo.toml" `
        --features tui `
        --release `
        --bin copypolish-tui
}
$tuiExe = Join-Path $RepoRoot "src-tauri\target\release\copypolish-tui.exe"
if (-not (Test-Path $tuiExe)) { throw "TUI build output was not found: $tuiExe" }
$tuiStaging = Join-Path $env:TEMP "copypolish-tui-windows-staging"
if (Test-Path $tuiStaging) { Remove-Item -Recurse -Force $tuiStaging }
New-Item -ItemType Directory -Force -Path $tuiStaging | Out-Null
Copy-Item $tuiExe (Join-Path $tuiStaging "CopyPolish-tui.exe")
$tuiArchive = Join-Path $dist "CopyPolish-tui-windows-x64.7z"
if (Test-Path $tuiArchive) { Remove-Item -Force $tuiArchive }
Push-Location $tuiStaging
try {
    & $sevenZip a -t7z -mx=9 $tuiArchive "CopyPolish-tui.exe"
    if ($LASTEXITCODE -ne 0) { throw "7-Zip (TUI) failed with exit code $LASTEXITCODE" }
}
finally {
    Pop-Location
}
Remove-Item -Recurse -Force $tuiStaging

& $python "$RepoRoot\scripts\verify_release_assets.py" $Tag --dist-dir $dist --platform windows
if ($LASTEXITCODE -ne 0) { throw "Windows asset verification failed" }
Remove-Item -Recurse -Force $staging
Get-ChildItem $dist
