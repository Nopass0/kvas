<#
.SYNOPSIS
Runs the required local validation gate before every commit.
.DESCRIPTION
This intentionally avoids GPU-only ignored tests, which require a compatible adapter.
Run those separately with ./tools/verify.ps1 -Gpu and report their hardware context.
#>
[CmdletBinding()]
param()

$ErrorActionPreference = 'Stop'
$workspaceRoot = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
Push-Location $workspaceRoot
try {
    $checks = @(
        @{ Name = 'Workspace compilation'; Args = @('check', '--workspace', '--all-targets') },
        @{ Name = 'Nested My Game scripts'; Args = @('check', '--manifest-path', 'games/Новая папка/My Game/scripts/Cargo.toml') }
    )

    foreach ($check in $checks) {
        Write-Host "`n== $($check.Name) =="
        & cargo @($check.Args)
        if ($LASTEXITCODE -ne 0) {
            throw "$($check.Name) failed with exit code $LASTEXITCODE. Fix the issue and rerun ./tools/pre-commit.ps1 before committing."
        }
    }

    Write-Host "`n== Changed Rust source formatting =="
    & (Join-Path $PSScriptRoot 'check-rustfmt.ps1')

    if (-not (Get-Command cargo-llvm-cov -ErrorAction SilentlyContinue)) {
        throw 'Install the coverage tool first: cargo install cargo-llvm-cov --locked'
    }

    Write-Host "`n== Workspace tests with line coverage =="
    $previousTargetDir = $env:CARGO_TARGET_DIR
    $previousCoverageTargetDir = $env:CARGO_LLVM_COV_TARGET_DIR
    $env:CARGO_TARGET_DIR = 'target/llvm-cov-target'
    $env:CARGO_LLVM_COV_TARGET_DIR = 'target/llvm-cov-target'
    try {
        & cargo llvm-cov --workspace --all-targets --lcov --output-path target/llvm-cov-target/lcov.info
        if ($LASTEXITCODE -ne 0) {
            throw "Workspace tests/coverage failed with exit code $LASTEXITCODE. Fix the issue and rerun ./tools/pre-commit.ps1 before committing."
        }
    }
    finally {
        $env:CARGO_TARGET_DIR = $previousTargetDir
        $env:CARGO_LLVM_COV_TARGET_DIR = $previousCoverageTargetDir
    }

    Write-Host "`n== Workspace doctests =="
    & cargo test --workspace --doc
    if ($LASTEXITCODE -ne 0) {
        throw "Workspace doctests failed with exit code $LASTEXITCODE. Fix the issue and rerun ./tools/pre-commit.ps1 before committing."
    }

    Write-Host "`nKvas pre-commit validation passed."
}
finally {
    Pop-Location
}
