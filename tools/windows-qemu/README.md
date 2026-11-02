# Nieuw OS — native Materiaal-testversie voor Windows/QEMU

Dit is een experimentele VM-testversie van de eigen x86_64-kernel met het
native `native-system-materiaal`- of `native-system-session-preview`-profiel
(zie `manifest.json`). De sessiepreview voegt herkenbare gebruikerskeuze en
een bureaublad toe; wachtwoord-/biometrische/FIDO2-authenticatie is daarin nog
niet gekoppeld. Alleen de expliciete VM-gasttest opent het bureaublad.
Het is geen installer, dagelijkse
release of bewijs dat Nieuw OS op een fysieke Windows-pc werkt. De meegeleverde
manifest.json bindt kernel, loader en firmware aan de bronversie.

## Starten

1. Installeer een 64-bit QEMU-build voor Windows vanaf de
   [officiële QEMU-downloadpagina](https://www.qemu.org/download/). De launcher
   vereist QEMU 10.2.1 of nieuwer, een SDL-schermbackend en `rng-builtin`.
2. Pak deze ZIP uit naar een eigen map. Open PowerShell in die map en start:

   ```powershell
   .\Start-NieuwOS.ps1 -QemuPath "C:\Program Files\qemu\qemu-system-x86_64.exe"
   ```

   Laat `-QemuPath` weg als `qemu-system-x86_64.exe` op PATH staat of in de
   standaardinstallatiemap is geïnstalleerd. Het script wijzigt de
   PowerShell-uitvoeringspolicy niet.
   Alleen de bundelhashes en QEMU-vereisten controleren, zonder een VM of
   `state\` aan te maken:

   ```powershell
   .\Start-NieuwOS.ps1 -CheckOnly
   ```

   De standaard `-Acceleration Auto` controleert of QEMU WHPX aanbiedt en
   initialiseert vóór de echte start een gepauzeerde testmachine zonder
   schijven, netwerk of gastuitvoering (maximaal 10 seconden). Bij succes
   gebruikt de echte VM WHPX; bij ontbreken/mislukken verschijnt een melding
   en volgt TCG-software-emulatie. `-CheckOnly` vraagt alleen de aangeboden
   backends op en test de initialisatie niet; er wordt dan geen VM gestart.
   Om hardwareversnelling verplicht te maken, zonder automatische fallback:

   ```powershell
   .\Start-NieuwOS.ps1 -Acceleration Whpx
   ```

   WHPX gebruikt CPU-virtualisatie via Windows Hypervisor Platform. Die
   Windows-functie en Intel VT-x/AMD-V in de firmware moeten beschikbaar
   zijn; wijzigingen daaraan gebeuren niet door dit script. Kies
   `-Acceleration Tcg` voor het oorspronkelijke softwareprofiel.
   De launcher bewaart de gekozen backend, reden en starttijd in
   `state\launch.json`. Een geslaagde initialisatieproef bewijst nog geen
   correcte Nieuw-OS-sessie. Na een fout in de echte VM wordt niet opnieuw
   gestart: dat voorkomt het ongemerkt herhalen van schijfbewerkingen.

   De SHA-256-controle detecteert beschadigde of gewijzigde bundelbestanden,
   maar is geen handtekening of bewijs van de afzender. Download alleen uit
   de afgesproken projectbron; vervang manifest of scripts niet door
   bestanden van onbekende herkomst.
3. Sluit de VM via het QEMU-venster. Opnieuw starten hergebruikt de
   VM-schijf en OVMF-variabelen uit `state\`. Om alleen de VM terug te zetten
   naar de lege beginstand: sluit QEMU volledig en verwijder
   `state\volume.img` en `state\OVMF_VARS.fd`; de launcher maakt ze opnieuw aan.

De VM gebruikt één vCPU (WHPX waar de initialisatie slaagt, anders TCG),
256 MiB RAM, een synthetische schrijfbare
documentimage en een read-only EFI-map. Er is geen netwerkkaart, direct
gekoppelde hostschijf, USB-doorvoer of clipboardkoppeling geconfigureerd. Via `fat:esp` krijgt de
gast wel read-only toegang tot de bundelbestanden in `esp\`. De map `state\`
bevat de schrijfbare VM-schijf, OVMF-variabelen en seriële uitvoer, maar wordt
niet als hostmap gekoppeld. Dit beschrijft alleen de geconfigureerde
QEMU-kanalen; het is geen garantie tegen kwetsbaarheden in QEMU.

## Grenzen en bewijsstatus

Dit image is bedoeld om de huidige native Materiaal-sessie te verkennen.
De profieldimensie is beperkt en de volledige product-/releaseacceptatie is
niet afgerond. Netwerk, audio, fysieke hardware, volledige desktopbediening,
toegankelijkheid en de complete opslag-/herstelmatrix zijn niet door deze
bundle bewezen. `Start-NieuwOS.ps1 -CheckOnly` en de vijf synthetische tests
van de eerdere `Test-Launcher.ps1` zijn op echte Windows PowerShell uitgevoerd; daarbij
start geen VM. VM-bedieningsbewijs op Windows of Linux is afzonderlijk en
geldt alleen voor de daarin vastgelegde bron, bundel en configuratie, niet
automatisch voor iedere nieuw verpakte ZIP.

De nieuwe versnellingsselectie is afzonderlijk met synthetische tests te
controleren: `./Test-Acceleration.ps1` vanuit de repositorymap
`tools/windows-qemu/`. Die test initialiseert geen hypervisor en meet geen
snelheid. Windows/WHPX-runtimebewijs voor deze wijziging staat nog open;
historische TCG-proeven gelden daarvoor niet. QEMU beschrijft beperkingen
voor legacy-VGA en SIMD-toegang tot MMIO; de launcher schakelt daarvoor geen
beveiligingsmaatregelen uit. Een modernere GPU-driver en minder
rendertransacties zijn afzonderlijke verbeteringen.

De firmwarebestanden worden met hun bijbehorende `licenses\`-copyrightnotices
meegeleverd. QEMU zelf wordt niet in deze ZIP opgenomen.

Bron voor Windows-binaries: QEMU Project, “Download QEMU”, geraadpleegd
30 oktober 2026, https://www.qemu.org/download/. De Windows-sectie verwijst
naar 64-bit binaries/installers van Stefan Weil en noemt MSYS2 als alternatief.

QEMU Project, “Windows Hypervisor Platform” en “Security”, ontwikkelhandleiding
11.1.50, geraadpleegd 2 november 2026:
https://www.qemu.org/docs/master/system/whpx.html en
https://www.qemu.org/docs/master/system/security.html.
