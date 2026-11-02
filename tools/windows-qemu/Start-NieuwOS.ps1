param(
    [string]$QemuPath,
    [ValidateSet("Auto", "Whpx", "Tcg")][string]$Acceleration = "Auto",
    [switch]$CheckOnly
)

$ErrorActionPreference = "Stop"

if (-not $QemuPath) {
    $qemuCommand = Get-Command "qemu-system-x86_64.exe" -ErrorAction SilentlyContinue
    if ($qemuCommand) {
        $QemuPath = $qemuCommand.Source
    } else {
        $candidate = Join-Path $env:ProgramFiles "qemu\qemu-system-x86_64.exe"
        if (Test-Path -LiteralPath $candidate -PathType Leaf) {
            $QemuPath = $candidate
        } else {
            throw "QEMU for Windows was not found. Pass -QemuPath with qemu-system-x86_64.exe."
        }
    }
}

if (-not (Test-Path -LiteralPath $QemuPath -PathType Leaf)) {
    throw "QEMU executable does not exist: $QemuPath"
}
$QemuPath = (Resolve-Path -LiteralPath $QemuPath).Path

$versionOutput = (& $QemuPath --version 2>&1 | Out-String)
if ($LASTEXITCODE -ne 0 -or $versionOutput -notmatch "QEMU emulator version ([0-9]+\.[0-9]+\.[0-9]+)") {
    throw "Could not read the QEMU version."
}
$qemuVersion = [version]$Matches[1]
if ($qemuVersion -lt [version]"10.2.1") {
    throw "QEMU 10.2.1 or later is required; found $qemuVersion."
}

$objectTypes = (& $QemuPath -object help 2>&1 | Out-String)
if ($LASTEXITCODE -ne 0 -or $objectTypes -notmatch "(?m)^\s*rng-builtin\s*$") {
    throw "This QEMU build does not provide rng-builtin; install a current 64-bit build."
}
$displayTypes = (& $QemuPath -display help 2>&1 | Out-String)
if ($LASTEXITCODE -ne 0 -or $displayTypes -notmatch "(?m)^\s*sdl\s*$") {
    throw "This QEMU build does not provide the SDL display backend."
}

$bundle = $PSScriptRoot
$manifestPath = Join-Path $bundle "manifest.json"
$manifest = Get-Content -LiteralPath $manifestPath -Raw | ConvertFrom-Json
if ($manifest.schema_version -ne 1 -or $manifest.kernel_feature -notin @("native-system-materiaal", "native-system-session-preview")) {
    throw "Not a supported native Materiaal test bundle."
}
$requiredFiles = @(
    "Start-NieuwOS.ps1", "Acceleration.ps1", "README.md",
    "OVMF_CODE.fd", "OVMF_VARS.template.fd", "bios-256k.bin", "VGA.rom",
    "licenses/OVMF-COPYRIGHT", "licenses/SeaBIOS-COPYRIGHT",
    "volume.empty.img", "esp/kernel.elf", "esp/EFI/BOOT/BOOTX64.EFI"
)
foreach ($relative in $requiredFiles) {
    $entry = $manifest.files.PSObject.Properties[$relative]
    if (-not $entry -or $entry.Value.sha256 -notmatch "^[0-9a-f]{64}$") {
        throw "Missing or malformed bundle hash: $relative"
    }
    $path = Join-Path $bundle $relative
    if (-not (Test-Path -LiteralPath $path -PathType Leaf)) {
        throw "Bundle file does not exist: $relative"
    }
    $item = Get-Item -LiteralPath $path -Force
    if ($item.PSIsContainer -or $item.Length -ne $entry.Value.size_bytes) {
        throw "Bundle file size differs: $relative"
    }
    if ((Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash -ne $entry.Value.sha256) {
        throw "Bundle file hash differs: $relative"
    }
}
$accelerators = (& $QemuPath -accel help 2>&1 | Out-String)
if ($LASTEXITCODE -ne 0) { throw "Could not query QEMU accelerators." }
# Load code only after it has passed the same manifest hash check as the loader.
. (Join-Path $bundle "Acceleration.ps1")
$selection = Select-NieuwOSAcceleration -Request $Acceleration -Available $accelerators -CheckOnly:$CheckOnly -Probe {
    Test-NieuwOSWhpx -QemuPath $QemuPath
}
Write-Output "Acceleration: $($selection.Name). $($selection.Reason)"
if ($selection.Name -eq "tcg" -and $Acceleration -eq "Auto") {
    Write-Warning "Hardware acceleration is unavailable; screen updates may be slow. See README.md for WHPX prerequisites."
}
if ($CheckOnly) {
    Write-Output "Bundle integrity and QEMU $qemuVersion prerequisites passed; no VM or mutable state created."
    return
}
$state = Join-Path $bundle "state"
New-Item -ItemType Directory -Path $state -Force | Out-Null
$vars = Join-Path $state "OVMF_VARS.fd"
$volume = Join-Path $state "volume.img"
if (-not (Test-Path -LiteralPath $vars -PathType Leaf)) {
    Copy-Item -LiteralPath (Join-Path $bundle "OVMF_VARS.template.fd") -Destination $vars
}
if (-not (Test-Path -LiteralPath $volume -PathType Leaf)) {
    Copy-Item -LiteralPath (Join-Path $bundle "volume.empty.img") -Destination $volume
}

Push-Location $bundle
try {
    $acceleratorArgument = if ($selection.Name -eq "whpx") { "whpx" } else { "tcg,thread=single" }
    $arguments = @(
        "-L", ".",
        "-machine", "pc-q35-8.2",
        "-accel", $acceleratorArgument,
        "-cpu", "qemu64",
        "-smp", "1",
        "-m", "256M",
        "-nodefaults",
        "-display", "sdl",
        "-monitor", "none",
        "-serial", "file:state/serial.txt",
        "-nic", "none",
        "-no-reboot",
        "-device", "VGA,vgamem_mb=16,romfile=VGA.rom",
        "-object", "rng-builtin,id=rng0",
        "-device", "virtio-rng-pci,rng=rng0,id=rng,addr=0x6,disable-legacy=on,max-bytes=1048576,period=1000",
        "-drive", "if=pflash,unit=0,format=raw,readonly=on,file=OVMF_CODE.fd",
        "-drive", "if=pflash,unit=1,format=raw,file=state/OVMF_VARS.fd",
        "-drive", "if=virtio,index=0,format=raw,readonly=on,file=fat:esp",
        "-drive", "if=none,id=volume,format=raw,cache=writeback,readonly=off,file=state/volume.img",
        "-device", "virtio-blk-pci,drive=volume,id=volblk,addr=0x5,disable-legacy=on,num-queues=1,queue-size=8,serial=NOS-SYSTEM-TEST"
    )
    $configuration = [ordered]@{
        requested_acceleration = $Acceleration
        selected_acceleration = $selection.Name
        acceleration_probe_passed = $selection.Checked
        reason = $selection.Reason
        qemu_version = $qemuVersion.ToString()
        machine = "pc-q35-8.2"
        cpu = "qemu64"
        vcpus = 1
        memory_mib = 256
        started_at_utc = [DateTime]::UtcNow.ToString("o")
    }
    $configuration | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $state "launch.json") -Encoding UTF8
    & $QemuPath @arguments
    if ($LASTEXITCODE -ne 0) {
        throw "QEMU exited with code $LASTEXITCODE. See state/serial.txt for guest output."
    }
} finally {
    Pop-Location
}
