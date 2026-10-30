# Nieuw OS — native Materiaal-testversie voor Windows/QEMU

Dit is een experimentele VM-testversie van de eigen x86_64-kernel met het
native `native-system-materiaal`-profiel. Het is geen installer, dagelijkse
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

   De SHA-256-controle detecteert beschadigde of gewijzigde bundelbestanden,
   maar is geen handtekening of bewijs van de afzender. Download alleen uit
   de afgesproken projectbron; vervang manifest of scripts niet door
   bestanden van onbekende herkomst.
3. Sluit de VM via het QEMU-venster. Opnieuw starten hergebruikt de
   VM-schijf en OVMF-variabelen uit `state\`. Om alleen de VM terug te zetten
   naar de lege beginstand: sluit QEMU volledig en verwijder
   `state\volume.img` en `state\OVMF_VARS.fd`; de launcher maakt ze opnieuw aan.

De VM gebruikt één TCG-vCPU, 256 MiB RAM, een synthetische schrijfbare
documentimage en een read-only EFI-map. Er is geen netwerkkaart, hostschijf,
hostmap, USB-doorvoer of clipboardkoppeling geconfigureerd. De gast kan dus
geen bestanden van de Windows-host openen via deze VM-configuratie.

## Grenzen en bewijsstatus

Dit image is bedoeld om de huidige native Materiaal-sessie te verkennen.
De profieldimensie is beperkt en de volledige product-/releaseacceptatie is
niet afgerond. Netwerk, audio, fysieke hardware, volledige desktopbediening,
toegankelijkheid en de complete opslag-/herstelmatrix zijn niet door deze
bundle bewezen. Het Windows-QEMU-startscript zelf is nog niet op een
Windows-host uitgevoerd; de project-QEMU-test op Linux is afzonderlijk bewijs.

De firmwarebestanden worden met hun bijbehorende `licenses\`-copyrightnotices
meegeleverd. QEMU zelf wordt niet in deze ZIP opgenomen.

Bron voor Windows-binaries: QEMU Project, “Download QEMU”, geraadpleegd
30 oktober 2026, https://www.qemu.org/download/. De Windows-sectie verwijst
naar 64-bit binaries/installers van Stefan Weil en noemt MSYS2 als alternatief.
