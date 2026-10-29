<#
.SYNOPSIS
Checks rustfmt only for changed Rust source files.
.DESCRIPTION
The repository contains existing unformatted legacy/generated sources. This keeps
the gate useful for new and modified authored code without causing unrelated churn.
Set RUSTFMT_BASE_REF in CI to check files changed relative to that Git ref.
#>
[CmdletBinding()]
param()

$ErrorActionPreference = 'Stop'
$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
Push-Location $repoRoot
try {
    $paths = @()
    if ($env:RUSTFMT_BASE_REF) {
        $paths += git diff --name-only "$($env:RUSTFMT_BASE_REF)...HEAD" -- '*.rs'
        if ($LASTEXITCODE -ne 0) { throw "Could not compare Rust files against $($env:RUSTFMT_BASE_REF)." }
    }
    else {
        $paths += git diff --cached --name-only --diff-filter=ACMR -- '*.rs'
        $paths += git diff --name-only --diff-filter=ACMR -- '*.rs'
        $paths += git ls-files --others --exclude-standard -- '*.rs'
    }

    $paths = @($paths | Where-Object { $_ -and $_ -notmatch '(^|/)generated/' } | Sort-Object -Unique)
    if ($paths.Count -eq 0) {
        Write-Host 'No authored Rust source files changed; rustfmt check skipped.'
        return
    }

    $rootManifest = Get-Content (Join-Path $repoRoot 'Cargo.toml') -Raw
    $workspaceEdition = if ($rootManifest -match '(?ms)^\[workspace\.package\].*?^edition\s*=\s*"([^"]+)"') { $Matches[1] } else { '2015' }
    $rustfmt = Get-Command rustfmt -ErrorAction Stop

    foreach ($relativePath in $paths) {
        $fullPath = Join-Path $repoRoot $relativePath
        if (-not (Test-Path -LiteralPath $fullPath -PathType Leaf)) { continue }

        $directory = Split-Path -Parent $fullPath
        $edition = $workspaceEdition
        while ($directory -and $directory.StartsWith($repoRoot, [System.StringComparison]::OrdinalIgnoreCase)) {
            $manifestPath = Join-Path $directory 'Cargo.toml'
            if (Test-Path -LiteralPath $manifestPath) {
                $manifest = Get-Content -LiteralPath $manifestPath -Raw
                if ($manifest -match '(?m)^edition\s*=\s*"([^"]+)"') { $edition = $Matches[1] }
                elseif ($manifest -match '(?m)^edition\.workspace\s*=\s*true') { $edition = $workspaceEdition }
                break
            }
            $parent = Split-Path -Parent $directory
            if ($parent -eq $directory) { break }
            $directory = $parent
        }

        Write-Host "rustfmt --check --edition $edition $relativePath"
        & $rustfmt.Source --check --edition $edition $fullPath
        if ($LASTEXITCODE -ne 0) { throw "rustfmt check failed for $relativePath." }
    }

    Write-Host "Checked formatting for $($paths.Count) changed authored Rust source file(s)."
}
finally {
    Pop-Location
}
