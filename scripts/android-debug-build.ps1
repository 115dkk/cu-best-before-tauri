<#
  Local Android debug build for Windows machines without symlink privilege.

  `tauri android build` compiles the Rust library fine but then fails while
  *symlinking* the .so into gen/android/app/src/main/jniLibs (needs Developer
  Mode or an elevated shell). This script runs the Tauri CLI for the compile
  step, copies the library instead, and finishes with Gradle while excluding
  the Tauri rust task so it does not try the symlink again.

  Usage:
    pwsh scripts/android-debug-build.ps1                # x86_64 (emulator)
    pwsh scripts/android-debug-build.ps1 -Arch aarch64  # arm64 device
    pwsh scripts/android-debug-build.ps1 -Install       # also adb install + launch
#>
[CmdletBinding()]
param(
    [ValidateSet('x86_64', 'aarch64')][string]$Arch = 'x86_64',
    [switch]$Install,
    [string]$Serial = ''
)

$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
Set-Location $root

if (-not $env:ANDROID_HOME) { $env:ANDROID_HOME = Join-Path $env:LOCALAPPDATA 'Android\Sdk' }
if (-not $env:NDK_HOME) {
    $ndk = Get-ChildItem (Join-Path $env:ANDROID_HOME 'ndk') -Directory | Sort-Object Name -Descending | Select-Object -First 1
    if (-not $ndk) { throw 'no NDK found under ANDROID_HOME/ndk' }
    $env:NDK_HOME = $ndk.FullName
}
if (-not $env:JAVA_HOME) { $env:JAVA_HOME = 'C:\Program Files\jdk-17' }

$map = @{
    'x86_64'  = @{ triple = 'x86_64-linux-android';  abi = 'x86_64';    flavor = 'X86_64' }
    'aarch64' = @{ triple = 'aarch64-linux-android'; abi = 'arm64-v8a'; flavor = 'Arm64' }
}
$t = $map[$Arch]

Write-Host "[1/3] compiling Rust library via Tauri CLI ($Arch, debug) ..."
# The CLI exits non-zero at the symlink step on this machine; that is expected.
& npx tauri android build --debug --target $Arch --apk 2>&1 | Where-Object { $_ -notmatch 'symbolic link|developer mode|SeCreateSymbolicLink|docs.microsoft.com|For Window' } | Out-Host

$so = Join-Path $root "target\$($t.triple)\debug\libcu_best_before_lib.so"
if (-not (Test-Path $so)) { throw "library not built: $so" }

Write-Host "[2/3] copying library into jniLibs/$($t.abi) ..."
$jni = Join-Path $root "src-tauri\gen\android\app\src\main\jniLibs\$($t.abi)"
New-Item -ItemType Directory -Force $jni | Out-Null
$dst = Join-Path $jni 'libcu_best_before_lib.so'
if (Test-Path $dst) { Remove-Item -Force $dst }
Copy-Item $so $dst

Write-Host "[3/3] gradle assemble$($t.flavor)Debug (rust task excluded) ..."
Push-Location (Join-Path $root 'src-tauri\gen\android')
try {
    & .\gradlew.bat "assemble$($t.flavor)Debug" -x "rustBuild$($t.flavor)Debug" --console=plain
    if ($LASTEXITCODE -ne 0) { throw "gradle failed with exit code $LASTEXITCODE" }
}
finally { Pop-Location }

$apk = Get-ChildItem (Join-Path $root 'src-tauri\gen\android\app\build\outputs\apk') -Recurse -Filter '*.apk' |
    Where-Object { $_.FullName -match 'debug' -and $_.FullName -match $t.abi.Replace('-', '.') -or $_.Name -match "$($t.flavor.ToLower())" } |
    Sort-Object LastWriteTime -Descending | Select-Object -First 1
if (-not $apk) {
    $apk = Get-ChildItem (Join-Path $root 'src-tauri\gen\android\app\build\outputs\apk') -Recurse -Filter '*debug*.apk' |
        Sort-Object LastWriteTime -Descending | Select-Object -First 1
}
if (-not $apk) { throw 'no APK produced' }
Write-Host "APK: $($apk.FullName)"

if ($Install) {
    $adb = Join-Path $env:ANDROID_HOME 'platform-tools\adb.exe'
    $sel = @(); if ($Serial) { $sel = @('-s', $Serial) }
    & $adb @sel install -r $apk.FullName
    & $adb @sel shell am start -n dev.dkk115.cubestbefore/.MainActivity
}
