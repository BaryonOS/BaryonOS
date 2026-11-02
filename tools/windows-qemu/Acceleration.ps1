# Selection policy for the interactive launcher. Native acceptance keeps its
# explicitly pinned TCG profile. No host settings or devices are changed here.
function Test-NieuwOSWhpxReply {
    param([AllowEmptyString()][string]$OutputText, [int]$ExitCode)
    $capabilities = $status = $quit = 0
    $invalidReply = $false
    foreach ($line in ($OutputText -split "`n")) {
        if (-not $line.Trim()) { continue }
        try { $reply = $line | ConvertFrom-Json } catch { $invalidReply = $true; continue }
        if ($reply.id -in @("cap", "probe-status", "probe-quit")) {
            if ($null -ne $reply.error -or $null -eq $reply.PSObject.Properties["return"] -or $null -eq $reply.return) {
                $invalidReply = $true
                continue
            }
            switch ($reply.id) {
                "cap" { $capabilities++ }
                "probe-status" {
                    if ($reply.return.status -in @("paused", "prelaunch")) { $status++ } else { $invalidReply = $true }
                }
                "probe-quit" { $quit++ }
            }
        }
    }
    return ($ExitCode -eq 0 -and -not $invalidReply -and $capabilities -eq 1 -and $status -eq 1 -and $quit -eq 1)
}

# Read one expected QMP handshake response under the shared ten-second budget.
# Events may interleave, but unknown IDs, errors and excessive events fail shut.
function Read-NieuwOSQmpReply {
    param($Reader, [System.Diagnostics.Stopwatch]$Clock, [string]$ExpectedId, [switch]$Greeting)
    for ($messages = 0; $messages -lt 32; $messages++) {
        $remaining = 10000 - [int]$Clock.ElapsedMilliseconds
        if ($remaining -le 0) { throw [System.TimeoutException]::new() }
        $lineTask = $Reader.ReadLineAsync()
        if (-not $lineTask.Wait($remaining)) { throw [System.TimeoutException]::new() }
        $line = $lineTask.GetAwaiter().GetResult()
        if ($null -eq $line -or $line.Length -gt 65536) { throw "QMP closed or oversized response." }
        $reply = $line | ConvertFrom-Json -ErrorAction Stop
        if ($Greeting) {
            if ($null -eq $reply.QMP.version -or $null -eq $reply.QMP.PSObject.Properties["capabilities"]) {
                throw "QMP greeting is missing."
            }
            return [pscustomobject]@{ Reply = $reply; Raw = $line }
        }
        if ($null -ne $reply.error) { throw "QMP rejected an initialization command." }
        if ($null -ne $reply.PSObject.Properties["id"]) {
            if ($reply.id -ne $ExpectedId -or $null -eq $reply.PSObject.Properties["return"] -or $null -eq $reply.return) {
                throw "QMP response does not match its request."
            }
            return [pscustomobject]@{ Reply = $reply; Raw = $line }
        }
        if ($null -eq $reply.PSObject.Properties["event"]) { throw "Unexpected QMP response." }
    }
    throw "Too many QMP events during initialization."
}

function Test-NieuwOSWhpx {
    param([Parameter(Mandatory = $true)][string]$QemuPath)

    $start = New-Object System.Diagnostics.ProcessStartInfo
    $start.FileName = $QemuPath
    # All arguments are fixed literals. -S prevents guest execution; this probe
    # has no disks, shared folders, network, USB or graphical window.
    $start.Arguments = "-no-user-config -machine pc-q35-8.2 -accel whpx -cpu qemu64 -smp 1 -m 256M -nodefaults -display none -monitor none -qmp stdio -serial none -nic none -S"
    $start.UseShellExecute = $false
    $start.CreateNoWindow = $true
    $start.RedirectStandardInput = $true
    $start.RedirectStandardOutput = $true
    $start.RedirectStandardError = $true
    $process = New-Object System.Diagnostics.Process
    $process.StartInfo = $start
    $started = $false
    $clock = [System.Diagnostics.Stopwatch]::StartNew()
    try {
        if (-not $process.Start()) { throw "Could not start the WHPX initialization probe." }
        $started = $true
        $stderr = $process.StandardError.ReadToEndAsync()
        $observed = New-Object 'System.Collections.Generic.List[string]'
        $greeting = Read-NieuwOSQmpReply -Reader $process.StandardOutput -Clock $clock -Greeting
        $observed.Add($greeting.Raw)
        # Wait for the real greeting and each reply before sending the next
        # command. Exact UTF-8 bytes with LF avoid text-newline translation.
        foreach ($request in @(
            @{ Json = '{"execute":"qmp_capabilities","id":"cap"}'; Id = "cap" },
            @{ Json = '{"execute":"query-status","id":"probe-status"}'; Id = "probe-status" },
            @{ Json = '{"execute":"quit","id":"probe-quit"}'; Id = "probe-quit" }
        )) {
            $requestBytes = [System.Text.Encoding]::UTF8.GetBytes($request.Json + "`n")
            $process.StandardInput.BaseStream.Write($requestBytes, 0, $requestBytes.Length)
            $process.StandardInput.BaseStream.Flush()
            $response = Read-NieuwOSQmpReply -Reader $process.StandardOutput -Clock $clock -ExpectedId $request.Id
            $observed.Add($response.Raw)
            if ($request.Id -eq "probe-status" -and $response.Reply.return.status -notin @("paused", "prelaunch")) {
                throw "QMP reported an unpaused machine."
            }
        }
        $remaining = 10000 - [int]$clock.ElapsedMilliseconds
        if ($remaining -le 0 -or -not $process.WaitForExit($remaining)) {
            throw [System.TimeoutException]::new()
        }
        $output = $observed -join "`n"
        $errorOutput = $stderr.GetAwaiter().GetResult()
        if (Test-NieuwOSWhpxReply -OutputText $output -ExitCode $process.ExitCode) {
            return [pscustomobject]@{ Ready = $true; Reason = "WHPX initialized with the paused q35 test profile." }
        }
        # The console may contain paths or device diagnostics; report only the
        # bounded QEMU text, never an environment or credential dump.
        $detail = ($errorOutput -replace "[\r\n]+", " ").Trim()
        if ($detail.Length -gt 240) { $detail = $detail.Substring(0, 240) }
        return [pscustomobject]@{ Ready = $false; Reason = "WHPX initialization failed (exit $($process.ExitCode)): $detail" }
    } catch [System.TimeoutException] {
        return [pscustomobject]@{ Ready = $false; Reason = "WHPX initialization timed out after 10 seconds." }
    } catch {
        return [pscustomobject]@{ Ready = $false; Reason = "WHPX initialization probe could not complete." }
    } finally {
        $clock.Stop()
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
