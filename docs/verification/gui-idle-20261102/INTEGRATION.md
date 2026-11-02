# Desktop en GUI-idle geïntegreerd — 2 november 2026

PR [123](https://github.com/simonvanieperen/nieuw-os/pull/123) is na review en
18 geslaagde CI-workflows opgenomen in de desktopbranch van
[122](https://github.com/simonvanieperen/nieuw-os/pull/122), mergecommit
`8289d60cd5caa8a26b7823a694858a15371dc2af`. Geen merge naar main of volledige
releaseacceptatie. De roadmap blijft **41/100**: de eerdere N01- en
U01-acceptatie brachten de score van 34 naar 41; de idlewijziging voegt geen
punten toe. U02 blijft open wegens klembord en volledige interactiebudgetten.

## Uitgevoerde controles

- Vier GUI-lussen gebruiken bestaande IPC_WAIT uitsluitend zonder onvoltooid
  werk. Primaire en twee gerichte AI-bronreviews vonden geen blocker.
  Kernel, rechten en transactiedeadlines zijn ongewijzigd. Contract:
  [GUI-IDLE-WAIT](../../implementation/GUI-IDLE-WAIT.md).
- Zes volledige Windows/WHPX-proeven, drie per arm, doorlopen alle dertien
  fasen inclusief echte compositorfault, herstel, opslaan, heropenen,
  onafhankelijke diskoracle en cleanup/raw exit 33. De mediane procesCPU
  daalt van 99,3724% naar 11,6663% van één logische processor: **88,2600% lager**.
  [Methode, originelen en grenzen](windows-ab/README.md).
- Root controleerde aggregate-/native-/serialhashes en berekende zowel de
  medianen als de ruwe Win32-tellers opnieuw met Decimal en eigen monotone
  meetintervallen. [Herberekening](root-cpu-recalculation.json),
  [centrale review](central-review.json). Dit is geen nieuwe Windows-run of
  menselijke audit. De oorspronkelijke beelden en volumes zijn lokaal door
  de bewijsreviewer gecontroleerd; root had die Windowsbestanden niet lokaal.
- Alle **18 CI-workflows** op head
  `2a7b74426027093a4793d78d5ddca94e15ae018a` slagen, inclusief algemene CI en
  gezamenlijke native systeemacceptatie. [Snapshot](ci-2a7b744-workflows.json).
- De echte [TCG-venstergate 41495583952](https://github.com/simonvanieperen/nieuw-os/actions/runs/41495583952)
  slaagt op CI-merge `a50216efcc5017443a4d5520998b95ac8047e1ae`: 323,900675 s,
  dertien fasen, native_gate=true en raw exit 33. De opgeslagen dertien
  fasebeelden, achttien hostorakels, ongesneden Materiaal-inhoud, beschermde
  randen, verplaatsing en herstelde berekening zijn opnieuw gecontroleerd.
  Het werkelijke volume bevat object 0, revisie 1, veertien bytes met
  SHA256 `9a45a00fa9ff69c17e124f54c14bb14cf2c7650e8cf1c272f6631f618a19a5a5`;
  canaries en cleanup zijn correct. Zie `ci-2a7b744-verification.json` en
  de gecomprimeerde oorspronkelijke native-/serialrapporten ernaast.
- Gerichte Rust-hosttests, strikte Clippy en fmt zijn herhaald op de
  productbron; na de Python-testfix slagen 181 tooltests. Gearchiveerde
  rootlogs staan onder `root-host/`. De pakketcontrole omvat 20 gerichte
  Pythonchecks, 22 launcherchecks en 33 QMP-helperchecks.

De CI draait Ubuntu/QEMU 8.2.2/TCG, q35-8.2, één CPU en 256 MiB. Dit is
afzonderlijk van de Windows/WHPX-metingen; verschillende run-/bouwomgevingen
vormen geen algemene benchmark of bewijs van identieke binaries.

## Bronversies behouden hun betekenis

| Uitvoering | Commit | Source-v3 |
|---|---|---|
| CPU-basis A | `337f2f29f1833ef4a8d2cd640b09419f85dbb83b` | `3919403542c923bc86467db3184345159a5ce6881567a0e11baa85f2d39f1995` |
| CPU-idle B | `1f2ac0ae7cf8944d77ca0656affb01fe9b086f1b` | `8faf3f7f47e3f26409e446b2311673416ddd70d5735ef106d94c9f30d2c6847a` |
| Finale CI en pakketbron | `2a7b74426027093a4793d78d5ddca94e15ae018a` | `084478cf4d2e137a3ba37bc6029716be3e38fecfdf76ca82fe55cd409b8536d0` |

Na B wijzigen alleen documentatie/bewijs, de CI-fetchvolgorde en één
Python-testbestand. Een willekeurige screenshot-UUID met “f12” kon de oude
substringassertie laten falen. De test inspecteert nu echte gestructureerde
QMP-keycodes, met regressies voor beide gevallen. De gewijzigde Python-input
verklaart de nieuwe volledige source-v3; alle gecompileerde OS-bronnen blijven
gelijk. De offline wipeharness houdt zijn offline/locked-controles; de workflow
haalt daarvoor eerst de gelockte workspaceafhankelijkheden op. Een cachehit
bewijst geen uitgevoerde cold-cachetest.

Loader, normale kernel en acht app-ELFs zijn expliciet bytegelijk aan de
eerder gebouwde productbron 8faf. Ook alle negentien pakketpayloads zijn gelijk;
alleen het brongebonden pakketmanifest verschilt. Zie
`package-2a7b744-binary-comparison.json`. Dit herlabelt de formele CPU-metingen
niet als nieuwe uitvoeringen. De finale CI heeft wel eigen native bewijs op
source0844; de CI-binaries verschillen van het afzonderlijk gebouwde pakket.

## Windows-testpakket

Normaal profiel `native-window-session`, loader `desktop-mode`,
`probe_controller=false`; geen crashinjectiecontroller. Bouw-/ELF-/licentie- en
negentien payloadcontroles: `package-2a7b744-verification.json`.

```text
/workspace/nieuw-os-windows-package/build/window-package-2a7b744/NieuwOS-venstersessie-2a7b744-Windows-QEMU.zip
SHA256 b685eb421e279da680da2c84bd280cc1edbc8684471f2019b2879c129c3ba1e0
```

Na uitpakken voert `Start-NieuwOS.ps1 -CheckOnly` de pakket-/QEMU-controles uit;
`Start-NieuwOS.ps1 -Acceleration Auto` kiest WHPX na een echte begrensde
QMP-initprobe, met zichtbare TCG-terugval. De pakketcheck hier gebruikt Linux
PowerShell met een synthetisch QEMU-help/versionbestand; zij start geen VM en
bewijst geen Windows-runtimeacceptatie van dit nieuwe normale pakket.

Het eerdere normale 337-CI-pakket is lokaal werkelijk gestart met PS5.1,
WHPX en SDL en na sluiten met exit 0 gestopt. De capturetool faalde met
0x800b0101; daarom blijft visuele/inputacceptatie van die normale bundel open.
Zie [afzonderlijk resultaat](normal-bundle-337/README.md). Dat verandert de
geslaagde volledige headless beeld-/invoerproeven niet. Geen hostinstellingen,
persoonlijke schijf, gedeelde hostmap of passthrough gewijzigd.

Dit levert geen echte authenticatie, algemene veiligheidsvergelijking,
energie-/temperatuurmeting, bewijs van afwezig redraw of harde 20ms-/p95-belofte.
De volledige releaseopdracht en fysieke/menselijke gates blijven behouden.
