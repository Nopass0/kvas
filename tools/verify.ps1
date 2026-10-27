<#
.SYNOPSIS
Checks the Kvas workspace, executable documentation and optional native GPU path.
.EXAMPLE
./tools/verify.ps1 -Gpu
.EXAMPLE
./tools/verify.ps1 -Release
#>
[CmdletBinding()]
param([switch]$Gpu, [switch]$Release)

$ErrorActionPreference = 'Stop'
$workspaceRoot = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
$previousDocFlags = $env:RUSTDOCFLAGS
$profileArguments = @()
if ($Release) { $profileArguments += '--release' }

function Invoke-Cargo {
    param([string[]]$CargoArguments)
    Write-Host ('cargo ' + ($CargoArguments -join ' '))
    & cargo @CargoArguments
    if ($LASTEXITCODE -ne 0) { throw "Cargo check failed with exit code $LASTEXITCODE" }
}

Push-Location $workspaceRoot
try {
    Invoke-Cargo -CargoArguments (@('check', '--workspace', '--all-targets') + $profileArguments)
    Invoke-Cargo -CargoArguments (@('test', '--workspace', '--all-targets') + $profileArguments)
    Invoke-Cargo -CargoArguments (@('test', '-p', 'kvas_ecs', '-p', 'kvas_engine', '-p', 'kvas_render', '--doc') + $profileArguments)
    $env:RUSTDOCFLAGS = (($previousDocFlags + ' -D warnings').Trim())
    Invoke-Cargo -CargoArguments @('doc', '-p', 'kvas_ecs', '-p', 'kvas_engine', '-p', 'kvas_render', '--no-deps')
    if ($Gpu) {
        Invoke-Cargo -CargoArguments (@('test', '-p', 'kvas_render', '--test', 'capture_gpu') + $profileArguments + @('--', '--ignored'))
    }
    Write-Host 'Kvas validation completed successfully.'
}
finally {
    $env:RUSTDOCFLAGS = $previousDocFlags
    Pop-Location
}
