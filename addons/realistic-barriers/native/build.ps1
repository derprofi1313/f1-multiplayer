# Builds native\realistic_barriers.dll with llvm-mingw (no Visual Studio needed), runs the host tests and packs
# the addon as dist\f1mp.realistic-barriers.swoftyaddon (install it on the launcher's Addons page).
# The SDK headers come from the app checkout this addon lives in: ..\..\..\packages\sdk\include
# Usage:  powershell -ExecutionPolicy Bypass -File build.ps1 [-Exe "C:\...\F1_25.exe"] [-NoTest] [-NoPack]
param(
    [string]$Exe = "",
    [switch]$NoTest,
    [switch]$NoPack
)
$ErrorActionPreference = "Stop"
$here = Split-Path -Parent $MyInvocation.MyCommand.Path
$addon = Split-Path -Parent $here
$sdk = Resolve-Path (Join-Path $here "..\..\..\packages\sdk\include")
$llvm = Join-Path $env:USERPROFILE "tools\llvm-mingw-20261020-ucrt-x86_64\bin"
$cxx = Join-Path $llvm "x86_64-w64-mingw32-clang++.exe"
if (-not (Test-Path $cxx)) { throw "llvm-mingw not found at $llvm" }
$env:PATH = "$llvm;$env:PATH"

# The version from swofty.json and a build stamp, so loader.log names the exact build.
$version = (Get-Content (Join-Path $addon "swofty.json") -Raw | ConvertFrom-Json).version
$commit = (& git -C $addon rev-parse --short=8 HEAD 2>$null)
if (-not $commit) { $commit = "local" }
$build = "build $((Get-Date).ToUniversalTime().ToString('yyyy-MM-dd')) $commit"

# A forced include rather than -D: Windows PowerShell mangles quotes inside native arguments.
$stamp = Join-Path $here "build\stamp.hpp"
$flags = @("-std=c++23", "-O2", "-Wall", "-Wextra", "-I$sdk", "-static", "-D__USE_MINGW_ANSI_STDIO=1", "-include", $stamp)
Push-Location $here
try {
    New-Item -ItemType Directory -Force build | Out-Null
    [IO.File]::WriteAllText($stamp, "#define RB_VERSION `"$version`"`n#define RB_BUILD `"$build`"`n")
    & $cxx @flags -shared -s -o realistic_barriers.dll src\barriers.cpp src\detour.cpp
    if ($LASTEXITCODE -ne 0) { throw "DLL build failed" }
    Remove-Item -ErrorAction SilentlyContinue realistic_barriers.lib, librealistic_barriers.dll.a
    Write-Host "built $here\realistic_barriers.dll ($version, $build)"

    if (-not $NoTest) {
        & $cxx @flags -g -o build\tests.exe test\tests.cpp
        if ($LASTEXITCODE -ne 0) { throw "test build failed" }
        if ($Exe -eq "") {
            $steam = "C:\Program Files (x86)\Steam\steamapps\common\F1 25\F1_25.exe"
            if (Test-Path $steam) { $Exe = $steam }
        }
        if ($Exe -ne "") { & build\tests.exe $Exe } else { & build\tests.exe }
        if ($LASTEXITCODE -ne 0) { throw "tests failed" }
    }

    if (-not $NoPack) {
        $python = Join-Path $env:LOCALAPPDATA "Programs\Python\Python312\python.exe"
        if (-not (Test-Path $python)) { $python = "python" }
        $pack = Resolve-Path (Join-Path $here "..\..\..\tools\pack_addon.py")
        & $python -I $pack $addon (Join-Path $addon "dist")
        if ($LASTEXITCODE -ne 0) { throw "packing failed" }
    }
} finally {
    Pop-Location
}
