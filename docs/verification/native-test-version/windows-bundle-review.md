# Windowsbundel: concrete reviewfixes

30 oktober 2026 (Europe/Amsterdam). Basis
`e6eecbc28bb4e74d1b1c7c01b26f2ab249cf4291`, branch
`copilot/windows-review-fixes-20261030`. Die basis bevat de afzonderlijk
beoordeelde stale-reportfix vanaf PR #66-head `c532ba8`.
Geteste gecombineerde werkboom: bronvingerafdruk v3
`24d5f4c112924cfe45681de3fcf3a92f62087e29524f15c74e14d83620891abd`.

## Implementatie

- Packager: genormaliseerd uitvoerpad; symlinktraversal geweigerd vóór en na
  het aanmaken van ouders. Nieuwe tests voor `..`, een intermediate symlink
  met ontbrekende leaf en vervanging door een symlink tijdens mkdir.
- Launcher: een ontbrekend bundelbestand geeft een eigen vaste foutmelding;
  de negatieve test matcht geen gelokaliseerde PowerShell-systeemmelding.
- Documentatie/manifest: `fat:esp` exposeert de eigen ESP read-only;
  `state` is backingopslag, geen gekoppelde hostmap. Dit is geen algemene
  QEMU-isolatiegarantie. Windowsvalidatie geldt per bron/bundel; CheckOnly
  en synthetische tests zijn geen VM-bediening.
- Historische previewrapporten behouden hun bytes en bestaande
  `source_commit`/`source_sha256`. Het ontbrekende
  `source_fingerprint_version` wordt niet alsnog als testbewijs geclaimd.

## Uitgevoerde controles

Vanuit de repositoryroot:

```sh
python3 -B -m unittest tools.tests.test_package_windows_qemu -v
python3 -B -m unittest tools.tests.test_materiaal_preview tools.tests.test_package_windows_qemu tools.tests.test_materiaal_probe
git diff --check
python3 -B tools/progress.py --write
python3 -B tools/progress.py --check
```

Gerichte packagertests: 10 geslaagd, exit 0. Gecombineerde suite na
fast-forward-integratie van de peerfix: 55 geslaagd, exit 0. De eerste
packagerrun eindigde met exit 1 door een inspringfout in de nieuwe test;
de fout is hersteld, de oorspronkelijke mislukte log blijft behouden.

Echte Windows PowerShell 5.1.26100.9756, met de bestaande QEMU 11.1.0 in
`C:\Program Files\qemu`. De gewijzigde eigen scripts zijn naar een unieke
lokale TEMP-map gekopieerd; geen uitvoeringspolicywijziging:

```sh
powershell.exe -NoProfile -Command '$ErrorActionPreference="Stop"; $source="\\wsl.localhost\Ubuntu\home\simon\.copilotd\worktrees\wsl-Ubuntu\nieuw-os\simonvanieperen-verbose-sniffle\tools\windows-qemu"; $root=Join-Path $env:TEMP ("NieuwOS-review-"+[guid]::NewGuid()); New-Item -ItemType Directory -Path $root | Out-Null; try { foreach($name in @("Start-NieuwOS.ps1","Test-Launcher.ps1")) { Copy-Item -LiteralPath (Join-Path $source $name) -Destination (Join-Path $root $name) }; & (Join-Path $root "Test-Launcher.ps1") -QemuPath "C:\Program Files\qemu\qemu-system-x86_64.exe"; if (-not $?) { throw "Launcher test failed" } } finally { foreach($name in @("Start-NieuwOS.ps1","Test-Launcher.ps1")) { $path=Join-Path $root $name; if(Test-Path -LiteralPath $path) { Remove-Item -LiteralPath $path } }; Remove-Item -LiteralPath $root }'
```

Exit 0: vijf tests geslaagd; twee CheckOnly-aanroepen, gemanipuleerde
kernel, ontbrekende kernel en geweigerd probeprofiel. Geen VM gestart;
synthetische bestanden en de stagingmap zijn opgeruimd.

Volledige lokale logs staan onder
`/home/simon/.copilot/session-state/d7a153299dbd45d7ab72b2596483b441/files/`:
`windows-review-package-tests.log` (mislukt),
`windows-review-package-tests-retry.log`,
`windows-review-integrated-tests.log` en
`windows-review-launcher-tests.log`.

## Bewijsgrens en rollback

Geen nieuwe native guestbuild/-run of volledige CI op deze gewijzigde
bron uitgevoerd. Het aparte Windows-interactiebewijs op eabc27b/c532
blijft bij die bron; geen menselijke UX-/hardwareacceptatie, cryptografische
bundelauthenticiteit of volledige productrelease geclaimd.
De padcontroles zijn geen bewijs tegen alle gelijktijdige
hostbestandssysteemaanvallen. Geen nieuwe rechten, afhankelijkheden of
kernelinterface. Rollback: uitsluitend deze tooling-/documentatiefix
terugdraaien, zonder VM-state of historische bewijsbestanden te wijzigen.
**34% → 34%**, geen nieuw rubriccriterium.
