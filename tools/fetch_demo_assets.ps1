[CmdletBinding()]
param()
$ErrorActionPreference = 'Stop'
$assetDir = Join-Path (Split-Path $PSScriptRoot -Parent) 'games/demo/assets/environments'
New-Item -ItemType Directory -Path $assetDir -Force | Out-Null
$file = Join-Path $assetDir 'studio_small_09_1k.hdr'
if (-not (Test-Path -LiteralPath $file)) {
    Invoke-WebRequest 'https://dl.polyhaven.org/file/ph-assets/HDRIs/hdr/1k/studio_small_09_1k.hdr' -OutFile $file
}
if ((Get-FileHash -LiteralPath $file -Algorithm MD5).Hash.ToLowerInvariant() -ne 'd5d7eb9d26d341d6aa4d11c1a54fd97c') { throw 'Poly Haven asset digest mismatch' }
Write-Host 'Powered by Poly Haven: Studio Small 09 by Sergej Majboroda, CC0. https://polyhaven.com/a/studio_small_09'
