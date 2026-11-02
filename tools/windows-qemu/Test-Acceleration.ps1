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
Write-Output "PASS: $passed acceleration policy tests; synthetic results only; no VM or hardware initialized."
