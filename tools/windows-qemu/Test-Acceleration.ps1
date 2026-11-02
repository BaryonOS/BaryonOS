$ErrorActionPreference = "Stop"
. (Join-Path $PSScriptRoot "Acceleration.ps1")
$passed = 0
$available = "Accelerators supported in QEMU binary:`nwhpx`ntcg`n"
$neverProbe = { throw "This selection must not start an initialization probe." }

function Assert-Name($Selection, [string]$Name, [bool]$Checked) {
    if ($Selection.Name -ne $Name -or $Selection.Checked -ne $Checked) {
        throw "Unexpected selection: $($Selection | ConvertTo-Json -Compress)"
    }
}
function Assert-Reject([scriptblock]$Action, [string]$Pattern) {
    $rejected = $false
    try { & $Action | Out-Null } catch {
        if ($_.Exception.Message -notlike $Pattern) { throw }
        $rejected = $true
    }
    if (-not $rejected) { throw "Expected rejection matching $Pattern" }
}

Assert-Name (Select-NieuwOSAcceleration -Available $available -CheckOnly -Probe $neverProbe) "whpx" $false
$passed++
Assert-Name (Select-NieuwOSAcceleration -Request Whpx -Available $available -CheckOnly -Probe $neverProbe) "whpx" $false
$passed++
Assert-Name (Select-NieuwOSAcceleration -Request Tcg -Available $available -Probe $neverProbe) "tcg" $false
$passed++
Assert-Name (Select-NieuwOSAcceleration -Available "tcg" -Probe $neverProbe) "tcg" $false
$passed++
Assert-Name (Select-NieuwOSAcceleration -Available $available -Probe { [pscustomobject]@{ Ready = $true; Reason = "synthetic success" } }) "whpx" $true
$passed++
Assert-Name (Select-NieuwOSAcceleration -Available $available -Probe { [pscustomobject]@{ Ready = $false; Reason = "synthetic initialization failure" } }) "tcg" $false
$passed++
Assert-Name (Select-NieuwOSAcceleration -Available $available -Probe { [pscustomobject]@{ Ready = $false; Reason = "synthetic timeout" } }) "tcg" $false
$passed++
Assert-Reject { Select-NieuwOSAcceleration -Request Whpx -Available "tcg" -Probe $neverProbe } "*does not list WHPX*"
$passed++
Assert-Reject { Select-NieuwOSAcceleration -Request Whpx -Available $available -Probe { [pscustomobject]@{ Ready = $false; Reason = "synthetic timeout" } } } "*synthetic timeout*"
$passed++
Assert-Reject { Select-NieuwOSAcceleration -Available "whpx" -Probe { [pscustomobject]@{ Ready = $false; Reason = "synthetic failure" } } } "*No TCG fallback*"
$passed++
Assert-Reject { Select-NieuwOSAcceleration -Request Tcg -Available "whpx" -Probe $neverProbe } "*does not provide TCG*"
$passed++
Assert-Name (Select-NieuwOSAcceleration -Available "not-whpx`ntcg" -Probe $neverProbe) "tcg" $false
$passed++

$greeting = '{"QMP":{"version":{"qemu":{"major":10,"minor":2,"micro":1}},"capabilities":[]}}'
$capabilityReply = '{"return":{},"id":"cap"}'
$statusReply = '{"return":{"status":"prelaunch","running":false},"id":"probe-status"}'
$quitReply = '{"return":{},"id":"probe-quit"}'
$validReplies = @($greeting, $capabilityReply, $statusReply, $quitReply) -join "`n"
foreach ($status in @("prelaunch", "paused")) {
    if (-not (Test-NieuwOSWhpxReply -OutputText ($validReplies.Replace("prelaunch", $status)) -ExitCode 0)) {
        throw "Valid paused QMP acknowledgements rejected: $status"
    }
    $passed++
}
foreach ($invalid in @(
    $greeting,
    (@($greeting, $statusReply, $quitReply) -join "`n"),
    (@($greeting, $capabilityReply, $statusReply) -join "`n"),
    $validReplies.Replace('"prelaunch"', '"running"'),
    $validReplies.Replace('"probe-status"', '"wrong-status"'),
    $validReplies.Replace($capabilityReply, '{"error":{"class":"GenericError"},"id":"cap"}'),
    $validReplies.Replace($quitReply, '{"return":null,"id":"probe-quit"}'),
    ($validReplies + "`n" + $statusReply),
    ($validReplies + "`nnot JSON"),
    ""
)) {
    if (Test-NieuwOSWhpxReply -OutputText $invalid -ExitCode 0) {
        throw "Incomplete, duplicate or invalid QMP acknowledgements accepted."
    }
    $passed++
}
if (Test-NieuwOSWhpxReply -OutputText $validReplies -ExitCode 1) {
    throw "Failed QEMU exit accepted as WHPX initialization success."
}
$passed++
foreach ($item in @(
    @{ Text = $greeting; Greeting = $true; Id = "" },
    @{ Text = $capabilityReply; Greeting = $false; Id = "cap" },
    @{ Text = ('{"event":"STOP"}' + "`n" + $statusReply); Greeting = $false; Id = "probe-status" }
)) {
    $reader = New-Object System.IO.StringReader($item.Text)
    $clock = [System.Diagnostics.Stopwatch]::StartNew()
    try {
        $reply = Read-NieuwOSQmpReply -Reader $reader -Clock $clock -Greeting:$item.Greeting -ExpectedId $item.Id
        if (-not $reply.Raw) { throw "QMP handshake did not return its expected response." }
        $passed++
    } finally { $reader.Dispose(); $clock.Stop() }
}
foreach ($item in @(
    @{ Text = $capabilityReply; Greeting = $true; Id = "" },
    @{ Text = $statusReply; Greeting = $false; Id = "cap" },
    @{ Text = '{"error":{"class":"GenericError"},"id":"cap"}'; Greeting = $false; Id = "cap" },
    @{ Text = (('{"event":"STOP"}' + "`n") * 32); Greeting = $false; Id = "cap" },
    @{ Text = ""; Greeting = $false; Id = "cap" }
)) {
    $reader = New-Object System.IO.StringReader($item.Text)
    $clock = [System.Diagnostics.Stopwatch]::StartNew()
    $rejected = $false
    try {
        try { Read-NieuwOSQmpReply -Reader $reader -Clock $clock -Greeting:$item.Greeting -ExpectedId $item.Id | Out-Null } catch { $rejected = $true }
        if (-not $rejected) { throw "Invalid QMP handshake response accepted." }
        $passed++
    } finally { $reader.Dispose(); $clock.Stop() }
}
Write-Output "PASS: $passed acceleration policy/QMP reply tests; synthetic results only; no VM or hardware initialized."
