# Selection policy for the interactive launcher. Native acceptance keeps its
# explicitly pinned TCG profile. No host settings or devices are changed here.
function Test-NieuwOSWhpx {
    param([Parameter(Mandatory = $true)][string]$QemuPath)

    $start = New-Object System.Diagnostics.ProcessStartInfo
    $start.FileName = $QemuPath
    # All arguments are fixed literals. -S prevents guest execution; this probe
    # has no disks, shared folders, network, USB or graphical window.
    $start.Arguments = "-machine pc-q35-8.2 -accel whpx -cpu qemu64 -smp 1 -m 256M -nodefaults -display none -monitor stdio -serial none -nic none -S"
    $start.UseShellExecute = $false
    $start.CreateNoWindow = $true
    $start.RedirectStandardInput = $true
    $start.RedirectStandardOutput = $true
    $start.RedirectStandardError = $true
    $process = New-Object System.Diagnostics.Process
    $process.StartInfo = $start
    $started = $false
    try {
        if (-not $process.Start()) { throw "Could not start the WHPX initialization probe." }
        $started = $true
        $stdout = $process.StandardOutput.ReadToEndAsync()
        $stderr = $process.StandardError.ReadToEndAsync()
        $process.StandardInput.WriteLine("info status")
        $process.StandardInput.WriteLine("quit")
        $process.StandardInput.Close()
        if (-not $process.WaitForExit(10000)) {
            $process.Kill()
            $process.WaitForExit()
            return [pscustomobject]@{ Ready = $false; Reason = "WHPX initialization timed out after 10 seconds." }
        }
        $output = $stdout.GetAwaiter().GetResult()
        $errorOutput = $stderr.GetAwaiter().GetResult()
        if ($process.ExitCode -eq 0 -and $output -match "VM status: (paused|prelaunch)") {
            return [pscustomobject]@{ Ready = $true; Reason = "WHPX initialized with the paused q35 test profile." }
        }
        # The console may contain paths or device diagnostics; report only the
        # bounded QEMU text, never an environment or credential dump.
        $detail = ($errorOutput -replace "[\r\n]+", " ").Trim()
        if ($detail.Length -gt 240) { $detail = $detail.Substring(0, 240) }
        return [pscustomobject]@{ Ready = $false; Reason = "WHPX initialization failed (exit $($process.ExitCode)): $detail" }
    } catch {
        return [pscustomobject]@{ Ready = $false; Reason = "WHPX initialization probe could not complete." }
    } finally {
        if ($started -and -not $process.HasExited) {
            $process.Kill()
            $process.WaitForExit()
        }
        $process.Dispose()
    }
}

function Select-NieuwOSAcceleration {
    param(
        [ValidateSet("Auto", "Whpx", "Tcg")][string]$Request = "Auto",
        [Parameter(Mandatory = $true)][string]$Available,
        [switch]$CheckOnly,
        [Parameter(Mandatory = $true)][scriptblock]$Probe
    )
    $hasWhpx = $Available -match "(?m)^\s*whpx\s*$"
    $hasTcg = $Available -match "(?m)^\s*tcg\s*$"
    if ($Request -eq "Tcg") {
        if (-not $hasTcg) { throw "This QEMU build does not provide TCG." }
        return [pscustomobject]@{ Name = "tcg"; Reason = "Software emulation explicitly selected."; Checked = $false }
    }
    if ($hasWhpx) {
        if ($CheckOnly) {
            return [pscustomobject]@{ Name = "whpx"; Reason = "WHPX is listed; initialization has not been tested by CheckOnly."; Checked = $false }
        }
        $result = & $Probe
        if ($result.Ready -eq $true) {
            return [pscustomobject]@{ Name = "whpx"; Reason = $result.Reason; Checked = $true }
        }
        $reason = $result.Reason
    } else {
        $reason = "This QEMU build does not list WHPX."
    }
    if ($Request -eq "Whpx") {
        throw "$reason Use -Acceleration Tcg to choose software emulation explicitly."
    }
    if (-not $hasTcg) { throw "$reason No TCG fallback is available." }
    return [pscustomobject]@{ Name = "tcg"; Reason = "$reason Falling back to slower software emulation."; Checked = $false }
}
