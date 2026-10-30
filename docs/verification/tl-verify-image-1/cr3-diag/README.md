# TASK-KS-CR3-DIAG — lege kernelmapping door ingestorte linkergrenzen

**Een enkele echte diagnostische gaststart reproduceert de bootfout.**
De directe oorzaak is afgebakend: de nieuwe CR3-tabellen bevatten voor
kernelcode, actieve stack en IDT geen aanwezige PD-entry. De exacte TCP-ELF
heeft `__kernel_start == __text_end == __rodata_end == __kernel_end ==
0x02000000`; daardoor is de bronlus die de kernel moet mappen leeg.
Dit is geen TCP-, NAS-, TLS-, allocator- of algemene pagingcorrectheidsclaim.

Datum 30 oktober 2026. Taak-ID `TASK-KS-CR3-DIAG`; rubric K02/K05/S08,
manageracceptatie-ID's FR-K02/O04/I01, zonder daarmee een nieuwe
productmatrixrij of aftekening te introduceren. Eigen checkoutbasis/HEAD
bij diagnose `a554bf2d14f3a6dfe2810f0259aa48e4be638617`,
branch `simon/tl-verify-image-1`. Alleen deze nieuwe bewijs-submap is
geschreven; kernel, loader, ABI, NAS, centrale status en eerdere
reproduceerbaarheidsrapporten zijn niet gewijzigd.

## Exacte bron en image

NET publiceerde ruwe rapporten op
`4a0ce99ff6b88adcd0e2c6be0571b30390d8d1c8`, runtimebron
`7af60df90468e47e2c551022b799bed37ea55b96`,
v3 `e659fbc8117587fccbccfc641e6e3730cf0980a2f665607fea1480a5a1711347`,
profiel `native-system-tcp`.
De gepubliceerde rapporten zijn als feitelijke input gelezen; de
sessieberichttool is niet beschikbaar, dus er is geen nieuwe mondelinge
bevestiging van NET geclaimd.

Alleen-lezen oorspronkelijke image:

```text
/home/simon/.copilotd/worktrees/wsl-Ubuntu/nieuw-os/simonvanieperen-musical-happiness/build/tls-port-worktree/build/tcp-main94-regression/run/system-normal-a06babbb1f89497ab60b59c514d0ac3c
```

Kernel SHA-256:
`10619b27ced3a57096817b52af3f11a0283029eebdee2446eda58fbc81659999`.
Loader SHA-256:
`0bfa2cbcccd921146b6de36cd4773e998dc85b0b1f718b5ca7bea5b4014ced59`.
Voor kopieren tegen het gepubliceerde rapport gecontroleerd; na de
diagnose beide oorspronkelijke bestanden opnieuw ongewijzigd bevonden.
Geen oude imagehash vervangen of byte-reproduceerbaarheid verondersteld.

Eigen diagnostische run: `build/ks-cr3-diag-run`.
De ESP en disk zijn gekopieerd; OVMF-VARS komt uit de bestaande eigen
ongewijzigde template met exact de oorspronkelijke templatehash.
De synthetische begindisk heeft de in het NET-rapport opgenomen beforehash.
Geen fysieke disk, persoonlijke inhoud of writes in de NET-worktree.

QEMU 10.2.1 / `pc-q35-10.0` / qemu64 / TCG single-thread / 1CPU / 256MiB.
Werkelijke executable:
`/tmp/nieuw-os-qemu-102/root/usr/bin/qemu-system-x86_64`,
SHA-256 `fa552a45c337a88fe71e1e1af9ebace875f30e88685a6986d9406c92efaf44cb`.
De volledige opdracht en `LD_LIBRARY_PATH` staan in `observation.json`.

## Waarneming vlak voor CR3

`observe.py` gebruikt QEMU's bestaande GDB-remote-interface rechtstreeks
via de Python-standaardbibliotheek; geen debuggerdependency geinstalleerd.
Een hardwarebreakpoint op `0x02001309` stopt voor `mov %rdi,%cr3`.
Geheugen/registers worden uitsluitend gelezen, met fysieke leesmodus voor
de tabellen. Geen `M`/`X`-geheugenwrites, registerwrites of gastpatches.
Daarna breakpoint verwijderen, debugger detach en de gast laten doorlopen.
Breakpoint/stop kunnen timing veranderen: dit is diagnose, geen
ongewijzigde normale acceptatierun.

| Pad | Waargenomen waarden |
|---|---|
| Nieuwe root, RDI | `0x1000` |
| RIP voor de write | `0x02001309` |
| Code `0x0200130c` | PML4[0]=`0x2003`, PDPT[0]=`0x3003`, PD[16]=0 |
| Actieve RSP `0x022e7ec0` | Dezelfde bovenlagen, PD[17]=0 |
| IDT `0x022f6038` | Dezelfde bovenlagen, PD[17]=0 |
| Rootpagina zelf `0x1000` | PD[0]=`0x6003`, leaf=`0x8000000000001003` |

De root is dus niet volledig leeg: de tabellen zijn wel gemapt, de
kernelrange niet. Na debugger detach volgt werkelijk
`#PF error=0x10`, RIP/CR2=`0x0200130c`, CR3=`0x1000`,
vervolgens double/triple fault. `fault-tail.txt` bewaart de trace.
Nieuwe QEMU raw exit 0 na triple fault is **geen geslaagde OS-exit**.
De seriele uitvoer eindigt bij KS-02; KS-03 wordt niet bereikt.
Diagnostische gastduur inclusief debuggerobservatie: zie `observation.json`.

De socket-netwerktegenpartij accepteert alleen de verbinding om dezelfde
device-topologie te kunnen booten. Zij implementeert geen TCP-protocoltest.
Deze diagnose stopt door de bootfout voordat de native netwerkproef kan
beginnen; geen groen netwerk-/coldresultaat.

## Gebonden ELF- en buildverklaring

`input-comparison.json` bewaart SHA-256, daadwerkelijke `nm`-uitvoer,
de beschikbare originele kernel-Cargo-fingerprint en beide configbestanden.
De werkelijke slechte ELF heeft alle vier grenssymbolen op `0x02000000`.
De kernelbron in `m2.rs` doet:

```text
for page in (text_start()..kernel_end).step_by(PAGE) { ... }
```

De disassembly van exact de slechte ELF (`kernel-mapping-loop.txt`) toont
de lengteberekening `0x02000000 - 0x02000000` en de overgeslagen lus.
Dat verklaart direct de fysiek waargenomen ontbrekende PD[16]/PD[17].
De IDT-fout daarna volgt eveneens uit de ontbrekende kernelmapping.

De originele Cargo-fingerprint toont **twee volledige kopieen** van de
target-rustflags, inclusief tweemaal `-C link-arg=-Tkernel/linker.ld`.
De buildcommand-cwd is een checkout onder:

```text
.../simonvanieperen-musical-happiness/build/tls-port-worktree
```

Zowel die checkout als de voorouder
`.../simonvanieperen-musical-happiness` bevat dezelfde `.cargo/config.toml`.
Cargo leest voorouderconfigs en voegt arraywaarden samen; dat past exact
bij de twee gemeten kopieen. De linkerbeschrijving zet de locatiecounter
aan het begin terug naar `0x02000000` en definieert de vier grenzen.
Dubbele scriptinvoer en samengevallen grenzen zijn hier werkelijk
vastgesteld; een afzonderlijke gecontroleerde linkerherbouw is niet
uitgevoerd, dus niet voorgesteld als reeds gevalideerde reparatie.

Vergelijking met bestaande eigen succesvolle 74d-Materiaal-artifact:

```text
__kernel_start  0x02000000
__text_end      0x02025000
__rodata_end    0x020e5000
__kernel_end    0x0250f000
```

Die andere ELF heeft niet de ingestorte grenzen en doorliep reeds beide
eigen functionele gates. `m2.rs`, `paging.rs`, `arch.rs` en `kernel/linker.ld`
zijn bytegelijk tussen die checkout en NET-bron 7af60df.
Dit is een gerichte bronvergelijking, geen image-identiteit of bewijs dat
alle overige buildinputs gelijk zijn.

Primaire referentie: The Cargo Book, *Configuration — Hierarchical
structure*, geraadpleegd 30 oktober 2026:
https://doc.rust-lang.org/cargo/reference/config.html#hierarchical-structure .
De documentatie vermeldt zoeken in huidige en alle bovenliggende mappen
en het samenvoegen van arrays. De eigen artifactmetingen, niet alleen
de documentatie, onderbouwen de dubbele flags en lege mapping.
CPU-context: Intel SDM volume 3, hoofdstuk 4 (paging) en 6 (exceptions);
`error=0x10` is hier niet-present supervisor instruction-fetch.

## Commands, mislukkingen en hervatstap

Werkelijk uitgevoerde diagnose:

```sh
python3 -B docs/verification/tl-verify-image-1/cr3-diag/observe.py
```

Exit 0 betekent observatie/archivering voltooid, **niet native TCP groen**.
De runner weigert een bestaande outputmap opnieuw te gebruiken.
Uitvoerlog `/tmp/nieuw-os-ks-cr3-diag-run.log`.
Volledige nieuwe CPU-trace lokaal:
`build/ks-cr3-diag-run/exceptions.log`.

Twee voorafgaande host-startpogingen bereikten geen gast:
zonder de bestaande verpakte libraryroute ontbrak `libjpeg.so.8`;
na herstel was de absolute Unix-socketnaam langer dan 108 bytes.
Beide scriptcommands exit 1, stderr bewaard als
`ks-cr3-diag-startup-error.txt` en `ks-cr3-diag-guest-startup-error.txt`.
De definitieve command gebruikt dezelfde aanwezige libraries en een
relatieve Unix-socketnaam, geen installaties of veiligheidsversoepelingen.
Daarom is slechts **een echte diagnostische gaststart** uitgevoerd.

**Kleinste voorgestelde herstelroute, nog niet uitgevoerd:** laat NET/build
dezelfde bron in een eigen siblingworktree buiten elke andere repo's
Cargo-configvoorouders bouwen, met eigen target/build; controleer dat de
kernel-fingerprint precies een `-Tkernel/linker.ld` bevat en
`start < text_end <= rodata_end < kernel_end`. Daarna de bestaande native
TCP-normal/coldgate uitvoeren. Niet voor deze fout kernelmappings verbreden,
NX uitzetten, IDT omzeilen of ABI wijzigen.

Eerstvolgend exact read-only commando op de vastgestelde input:

```sh
nm -n /home/simon/.copilotd/worktrees/wsl-Ubuntu/nieuw-os/simonvanieperen-musical-happiness/build/tls-port-worktree/build/tcp-main94-regression/run/system-normal-a06babbb1f89497ab60b59c514d0ac3c/esp/kernel.elf | grep -E '__kernel_start|__text_end|__rodata_end|__kernel_end'
```

Voor de echte herbouw moet NET/release eerst de concrete siblingworktree
en eigen doelmappen afspreken; geen runtimepatchclaim door deze sessie.
Een structurele build-/ELF-inspectorgate voor geordende grenssymbolen is
eventueel apart aan buildteam toe te wijzen, niet hier stil geimplementeerd.
S09 blijft bij manager in wachtrij. Geen scorepunten; voorstel 34% -> 34%.
