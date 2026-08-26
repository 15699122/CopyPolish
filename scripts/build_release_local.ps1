# Windows 本地构建与 .7z 打包（roadmap §3 / docs/manual-release.md）。
#
# 约束：
# - 必须在干净的发布工作区运行（git status 无任何改动）；
# - 不创建 tag、不推送、不上传 Release；产物写入被忽略的 dist\；
# - Windows 资产必须在 Windows 上构建（本项目不配置交叉编译）；
# - 需要 7-Zip CLI（7z.exe）在 PATH 中。
#
# 用法（在仓库根目录的 PowerShell 中）：
#   .\scripts\build_release_local.ps1 <vX.Y.Z[-suffix]> [-SkipVerify]
#
# 示例：
#   .\scripts\build_release_local.ps1 v0.5.0

param(
    [Parameter(Mandatory = $true)]
    [string]$Tag,
    [switch]$SkipVerify
)

$ErrorActionPreference = "Stop"
$RepoRoot = Split-Path -Parent (Split-Path -Parent $PSCommandPath)
Set-Location $RepoRoot

Write-Host "== 检查干净发布工作区 =="
$dirty = git status --porcelain
if ($dirty) {
    Write-Error "工作区存在未提交改动。请在独立的发布 worktree 中执行（见 docs/manual-release.md 第 3 节）。"
}

Write-Host "== 同步 tag 完整版本 ($Tag) =="
python3 "$RepoRoot\scripts\prepare_release_version.py" $Tag
python3 "$RepoRoot\scripts\check_version.py" $Tag

if (-not $SkipVerify) {
    Write-Host "== 发布前统一验证（与 CI 对齐）=="
    npm ci --prefix frontend
    npm test --prefix frontend -- --run
    npm run build --prefix frontend
    cargo fmt --manifest-path src-tauri/Cargo.toml --check
    cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
    cargo test --manifest-path src-tauri/Cargo.toml
    git diff --check
}
else {
    Write-Host "== 跳过验证（-SkipVerify）=="
}

Write-Host "== 构建 Windows 便携 exe =="
npm ci --prefix frontend
npm run tauri --prefix frontend -- build -- --no-bundle

$ExePath = "src-tauri\target\release\chinese-copywriting-formatter.exe"
if (-not (Test-Path $ExePath)) {
    Write-Error "找不到构建产物: $ExePath"
}

Write-Host "== 收集 exe 与旁置 DLL 到 staging 根目录 =="
$Staging = Join-Path $PWD "windows-portable-staging"
if (Test-Path $Staging) { Remove-Item -Recurse -Force $Staging }
New-Item -ItemType Directory -Path $Staging | Out-Null

Copy-Item $ExePath (Join-Path $Staging "CopyPolish.exe")

# 与 release.yml 一致：构建输出同目录存在的旁置 DLL 一并复制进 staging 根目录。
$BuildDir = Split-Path -Parent $ExePath
Get-ChildItem -Path $BuildDir -Filter *.dll | ForEach-Object {
    Copy-Item $_.FullName (Join-Path $Staging $_.Name)
}

Write-Host "== 在 staging 目录内部压缩为 .7z（根目录直接包含 exe）=="
$SevenZip = Get-Command 7z.exe -ErrorAction SilentlyContinue
if (-not $SevenZip) {
    Write-Error "未找到 7z.exe。请安装 7-Zip 并确保其在 PATH 中。"
}
$DistDir = Join-Path $PWD "dist"
New-Item -ItemType Directory -Force -Path $DistDir | Out-Null
$Archive = Join-Path $DistDir "CopyPolish-windows-x64.7z"
if (Test-Path $Archive) { Remove-Item -Force $Archive }

Push-Location $Staging
try {
    & 7z.exe a -t7z $Archive * | Out-Host
    if ($LASTEXITCODE -ne 0) { Write-Error "7z 压缩失败，退出码 $LASTEXITCODE" }
}
finally {
    Pop-Location
}

Copy-Item (Join-Path $Staging "CopyPolish.exe") (Join-Path $DistDir "CopyPolish.exe")
Remove-Item -Recurse -Force $Staging

Write-Host "== 校验产物 =="
python3 "$RepoRoot\scripts\verify_release_assets.py" $Tag --dist-dir dist
if ($LASTEXITCODE -ne 0) {
    Write-Host "提示：Linux 资产缺失属正常现象——本脚本仅产出 Windows 资产，" 
    Write-Host "完整五资产校验需在 Linux 构建完成后合并目录再跑一次。"
}

Write-Host "完成：Windows 资产已输出到 $DistDir"
Get-ChildItem $DistDir
