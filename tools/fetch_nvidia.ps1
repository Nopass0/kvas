[CmdletBinding()]
param()
$ErrorActionPreference = 'Stop'
$sdkRoot = Join-Path (Split-Path $PSScriptRoot -Parent) 'third_party/nvidia'
New-Item -ItemType Directory -Path $sdkRoot -Force | Out-Null
$archive = Join-Path $sdkRoot 'streamline-sdk-v2.14.16.zip'
$digest = '32041e51db0cab709a271b72a352281b690c2abbd669a827809401a5dd23956b'
if (-not (Test-Path -LiteralPath $archive)) {
    Invoke-WebRequest 'https://github.com/NVIDIA-RTX/Streamline/releases/download/v2.14.16/streamline-sdk-v2.14.16.zip' -OutFile $archive
}
if ((Get-FileHash -LiteralPath $archive -Algorithm SHA256).Hash.ToLowerInvariant() -ne $digest) { throw 'Streamline SHA256 mismatch; archive retained for inspection.' }
$destination = Join-Path $sdkRoot 'streamline-v2.14.16'
if (-not (Test-Path -LiteralPath $destination)) { Expand-Archive -LiteralPath $archive -DestinationPath $destination }
$neural = Join-Path $sdkRoot 'RTXNS'
if (-not (Test-Path -LiteralPath $neural)) {
    & git clone --depth 1 --branch v1.5.0 https://github.com/NVIDIA-RTX/RTXNS $neural
    if ($LASTEXITCODE -ne 0) { throw 'RTXNS clone failed' }
}
$revision = & git -C $neural rev-parse HEAD
if ($LASTEXITCODE -ne 0 -or $revision -ne 'f309ae68677a67060cc6ded2d0af925a3931435f') { throw 'RTXNS revision mismatch; checkout was not modified.' }
Write-Host "SDKs verified at $sdkRoot. See docs/NVIDIA.md for integration status."
