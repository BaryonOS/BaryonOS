param(
    [Parameter(Mandatory = $true)]
    [string]$QemuPath
)

$ErrorActionPreference = "Stop"
$root = Join-Path ([System.IO.Path]::GetTempPath()) ("Nieuw OS launcher test " + [guid]::NewGuid())
$notoNotices = @(
    "licenses/NotoSansMono-NOTICE.md", "licenses/NotoSansMono-OFL.txt",
    "licenses/noto-sans-mono-bitmap-MIT.txt"
)
$files = @(
    "Start-NieuwOS.ps1", "Acceleration.ps1", "README.md",
    "OVMF_CODE.fd", "OVMF_VARS.template.fd", "bios-256k.bin", "VGA.rom",
    "licenses/OVMF-COPYRIGHT", "licenses/SeaBIOS-COPYRIGHT",
    "licenses/OpenSans-NOTICE.md", "licenses/OpenSans-Apache-2.0.txt",
    "licenses/Native-graphics-README.md", "licenses/OpenSans-font-provenance.json",
    "volume.empty.img", "esp/kernel.elf", "esp/EFI/BOOT/BOOTX64.EFI"
) + $notoNotices
$passed = 0
try {
    New-Item -ItemType Directory -Path $root | Out-Null
    foreach ($relative in $files) {
        $path = Join-Path $root $relative
        New-Item -ItemType Directory -Path (Split-Path $path) -Force | Out-Null
        if ($relative -eq "Start-NieuwOS.ps1" -or $relative -eq "Acceleration.ps1") {
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

    $helper = Join-Path $root "Acceleration.ps1"
    $helperOriginal = [System.IO.File]::ReadAllBytes($helper)
    $helperChanged = $helperOriginal.Clone()
    $helperChanged[0] = $helperChanged[0] -bxor 1
    [System.IO.File]::WriteAllBytes($helper, $helperChanged)
    $rejected = $false
    try { & $launcher -QemuPath $QemuPath -CheckOnly } catch {
        if ($_.Exception.Message -ne "Bundle file hash differs: Acceleration.ps1") { throw }
        $rejected = $true
    }
    if (-not $rejected) { throw "Tampered acceleration code was accepted." }
    $passed++
    [System.IO.File]::WriteAllBytes($helper, $helperOriginal)

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
    $manifest.kernel_feature = "native-window-session"
    $manifest.loader_feature = "desktop-mode"
    $manifest.session = @{
        classification = "public-workspace-research"
        authentication_integrated = $false
        initial_processes = 8
        width = 1280
        height = 720
    }
    $manifest | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath $manifestPath -Encoding UTF8
    & $launcher -QemuPath $QemuPath -CheckOnly
    if (Test-Path -LiteralPath (Join-Path $root "state")) {
        throw "Window CheckOnly created mutable VM state."
    }
    $passed++
    $manifest.session.authentication_integrated = $true
    $manifest | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath $manifestPath -Encoding UTF8
    $rejected = $false
    try { & $launcher -QemuPath $QemuPath -CheckOnly } catch {
        if ($_.Exception.Message -notlike "*authentication is not integrated*") { throw }
        $rejected = $true
    }
    if (-not $rejected) { throw "False authentication claim was accepted." }
    $passed++
    $manifest.session.authentication_integrated = $false
    foreach ($profile in @("native-window-session-probe", "native-window-session-hidpi")) {
        $manifest.kernel_feature = $profile
        $manifest | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath $manifestPath -Encoding UTF8
        $rejected = $false
        try { & $launcher -QemuPath $QemuPath -CheckOnly } catch {
            if ($_.Exception.Message -notlike "*Not a supported*") { throw }
            $rejected = $true
        }
        if (-not $rejected) { throw "Unsupported window profile was accepted." }
        $passed++
    }
    foreach ($profile in @("native-system-materiaal", "native-window-session")) {
        $manifest.kernel_feature = $profile
        $manifest | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath $manifestPath -Encoding UTF8
        foreach ($relative in $notoNotices) {
            $notice = Join-Path $root $relative
            $noticeOriginal = [System.IO.File]::ReadAllBytes($notice)
            Remove-Item -LiteralPath $notice
            $rejected = $false
            try { & $launcher -QemuPath $QemuPath -CheckOnly } catch {
                if ($_.Exception.Message -ne "Bundle file does not exist: $relative") { throw }
                $rejected = $true
            }
            if (-not $rejected) { throw "$profile accepted missing notice: $relative" }
            $passed++
            [System.IO.File]::WriteAllBytes($notice, $noticeOriginal)

            $noticeChanged = $noticeOriginal.Clone()
            $noticeChanged[0] = $noticeChanged[0] -bxor 1
            [System.IO.File]::WriteAllBytes($notice, $noticeChanged)
            $rejected = $false
            try { & $launcher -QemuPath $QemuPath -CheckOnly } catch {
                if ($_.Exception.Message -ne "Bundle file hash differs: $relative") { throw }
                $rejected = $true
            }
            if (-not $rejected) { throw "$profile accepted changed notice: $relative" }
            $passed++
            [System.IO.File]::WriteAllBytes($notice, $noticeOriginal)
        }
    }
    if (Test-Path -LiteralPath (Join-Path $root "state")) {
        throw "Notice integrity checks created mutable VM state."
    }
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
