param(
    [Parameter(Mandatory = $true)]
    [string]$QemuPath
)

$ErrorActionPreference = "Stop"
$root = Join-Path ([System.IO.Path]::GetTempPath()) ("Nieuw OS launcher test " + [guid]::NewGuid())
$files = @(
    "Start-NieuwOS.ps1", "README.md",
    "OVMF_CODE.fd", "OVMF_VARS.template.fd", "bios-256k.bin", "VGA.rom",
    "licenses/OVMF-COPYRIGHT", "licenses/SeaBIOS-COPYRIGHT",
    "volume.empty.img", "esp/kernel.elf", "esp/EFI/BOOT/BOOTX64.EFI"
)
$passed = 0
try {
    New-Item -ItemType Directory -Path $root | Out-Null
    foreach ($relative in $files) {
        $path = Join-Path $root $relative
        New-Item -ItemType Directory -Path (Split-Path $path) -Force | Out-Null
        if ($relative -eq "Start-NieuwOS.ps1") {
            Copy-Item -LiteralPath (Join-Path $PSScriptRoot $relative) -Destination $path
        } else {
            [System.IO.File]::WriteAllText($path, "Synthetic launcher test: $relative")
        }
    }
    $records = @{}
    foreach ($relative in $files) {
        $path = Join-Path $root $relative
        $records[$relative] = @{
            sha256 = (Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash.ToLowerInvariant()
            size_bytes = (Get-Item -LiteralPath $path).Length
        }
    }
    $manifest = @{
        schema_version = 1
        kernel_feature = "native-system-materiaal"
        files = $records
    }
    $manifestPath = Join-Path $root "manifest.json"
    $manifest | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath $manifestPath -Encoding UTF8
    $launcher = Join-Path $root "Start-NieuwOS.ps1"

    & $launcher -QemuPath $QemuPath -CheckOnly
    if (Test-Path -LiteralPath (Join-Path $root "state")) {
        throw "CheckOnly created mutable VM state."
    }
    $passed++
    & $launcher -CheckOnly
    $passed++

    $kernel = Join-Path $root "esp/kernel.elf"
    $original = [System.IO.File]::ReadAllBytes($kernel)
    $changed = $original.Clone()
    $changed[0] = $changed[0] -bxor 1
    [System.IO.File]::WriteAllBytes($kernel, $changed)
    $rejected = $false
    try { & $launcher -QemuPath $QemuPath -CheckOnly } catch {
        if ($_.Exception.Message -notlike "*hash differs*") { throw }
        $rejected = $true
    }
    if (-not $rejected) { throw "Tampered same-size kernel was accepted." }
    $passed++
    [System.IO.File]::WriteAllBytes($kernel, $original)

    Remove-Item -LiteralPath $kernel
    $rejected = $false
    try { & $launcher -QemuPath $QemuPath -CheckOnly } catch {
        if ($_.Exception.Message -ne "Bundle file does not exist: esp/kernel.elf") { throw }
        $rejected = $true
    }
    if (-not $rejected) { throw "Missing kernel was accepted." }
    $passed++
    [System.IO.File]::WriteAllBytes($kernel, $original)

    $manifest.kernel_feature = "native-system-materiaal-probe"
    $manifest | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath $manifestPath -Encoding UTF8
    $rejected = $false
    try { & $launcher -QemuPath $QemuPath -CheckOnly } catch {
        if ($_.Exception.Message -notlike "*Not a supported*") { throw }
        $rejected = $true
    }
    if (-not $rejected) { throw "Automated probe was accepted as an interactive bundle." }
    $passed++
    Write-Output "PASS: $passed Windows launcher tests; synthetic files only; no VM started."
} finally {
    foreach ($relative in $files) {
        $path = Join-Path $root $relative
        if (Test-Path -LiteralPath $path -PathType Leaf) { Remove-Item -LiteralPath $path }
    }
    if (Test-Path -LiteralPath (Join-Path $root "manifest.json")) {
        Remove-Item -LiteralPath (Join-Path $root "manifest.json")
    }
    foreach ($relative in @("esp/EFI/BOOT", "esp/EFI", "esp", "licenses", "")) {
        $path = if ($relative) { Join-Path $root $relative } else { $root }
        if (Test-Path -LiteralPath $path -PathType Container) { Remove-Item -LiteralPath $path }
    }
}
