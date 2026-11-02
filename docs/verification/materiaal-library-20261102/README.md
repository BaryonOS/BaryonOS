# U03 — native Materiaalbibliotheek geaccepteerd

**41 → 44/100**, uitsluitend de vaste drie punten voor U03. De criteria,
gewichten en eindscope zijn ongewijzigd. [Concept-PR 130](https://github.com/simonvanieperen/nieuw-os/pull/130)
staat boven PR 129; er is geen merge naar main of volledige releaseclaim.

- Branch: `codex/materiaal-library-20261102`.
- Begin: `159ad799d2a94fb6542d60e26ed8bbd22bdbd752`.
- Native geteste code: `64044318c622eaacab25272611acf856a5fca5d5`.
- Source-v3: `069666e5406a893f64aa5d8e00298863f33f8be9cef57b6e0522b8bdf9648670`.

## Wat werkelijk is gebouwd

De eigen Rust-kernel en UEFI-loader draaien Materiaal en Rekenmachine als
aparte CPL3-processen. Materiaal heeft nu leesbare proportionele tekst, een
prominente documenttitel, vaste bewerkingsacties en echte locatie-, rechten- en
revisiecontext. Tekst, caret, selectie, wrapping, pointer en navigatie gebruiken
dezelfde glyphmetingen, inclusief de gekozen kant van een zachte regelomloop.
Opgeslagen UTF-8-bytes worden niet aangepast voor de weergave.

Gebruikers kunnen een document maken, typen, hernoemen en bewaren, lokaal op
naam zoeken, het in een bestaande of nieuwe collectie ordenen, eerdere inhoud
bekijken en die als nieuwe revisie herstellen. Een verse disk toont een lege
bibliotheek. De QMP-proef typt de notitie zelf; de lijst en geschiedenis komen
uit de echte volume-v7-dienst.

De SDK-wrappers behouden domeinbinding, read-only, CAS, dirty/busy/conflict en
onzekere publicatie. Mutaties worden niet blind herhaald. Bij een bevestigde
restore met mislukte herlezing blokkeert bewaren tot expliciet heropenen. Een
begrensde tekstpreview valideert eerst de gehele historische versie en UTF-8
in de bestaande stagingbuffer. Verkorting en scrollbereik zijn zichtbaar.

Geen nieuw kernelrecht, wireformaat, opslagformaat, dependencyversie of unsafe.
Het native volumeprofiel heeft acht objecten per verleend domein, vier
collecties, maximaal 64 KiB per document en twee historische versies.

## Native acceptatie

| Proef | Fasen | Werkelijk resultaat | Duur |
|---|---:|---|---:|
| Bibliotheek schrijven | 10 | raw 33, library gate | 382,604 s |
| Koude heropening | 4 | raw 33, bytegelijke kopie | 191,301 s |
| Clipboard/compositorregressie | 22 | raw 33, native en clipboard gates | 487,007 s |

[Write](native-write.json), [cold](native-cold.json),
[regressie](native-regression.json), [onafhankelijke readback-index](readback-index.json).
Alle drie proeven zijn afzonderlijke VM's en onafhankelijk teruggelezen.
Een library gate vervangt de compositor-/clipboardgate niet.

De writeproef vergelijkt tien complete toegewezen appbeelden met actuele
Rust-hostoracles, zonder masks. De onafhankelijke diskdecoder leest werkelijk
object 0, revisie 4/current 4/history 3, naam `Reisnotities`, collectie `Reis`
met revisie 1 en beide exacte UTF-8-versies. Generatie 5, domeinklokken [5,0],
canaries en alle andere objecten/collecties zijn gecontroleerd.

De coldproef valideert de parent, bron, configuratie, serial en beelden opnieuw.
De geïsoleerde kopie blijft bytegelijk, SHA256
`31a367e7c499a8a1e21e954b3328d2a08ccf02ea2b5c89ce0a114213e19467ea`.
Naam, inhoud, collectie en geschiedenis blijven behouden; er is geen save-ACK.

De volledige regressie controleert een echte UD2-fault in slot 4/vector 6,
compositor PID 5 → 9, behoud van apps 4/8 en hun werk, verse epochs en
probes 3 → 10. Vier Copy-, vijf Paste- en één Clear-actie, post-fault Empty,
14 bytes/revisie 1 op disk, canaries en volledige cleanup slagen. Opnieuw
gecontroleerd: 22 framehashes, 34 hostoracles, 17 complete appbodymatches en
twee calculatorverplaatsings-/herstelgelijkheden. Het snapshotveld `REVISION`
betekent laatste gewone save-ACK; de huidige revisie na META/RESTORE blijkt
uit het exacte beeld en de onafhankelijke diskoracle.

Rawserial: [write](native-write-serial.txt.gz), [cold](native-cold-serial.txt.gz),
[regressie](native-regression-serial.txt.gz). Compressie is verliesloos.

## Visuele beoordeling en bewijsniveaus

[Voor: native 703](native/before-703-desktop.png),
[na: bewaren](native/library-saved.png), [geschiedenis](native/library-history.png),
[hersteld](native/library-restored.png), [zoeken](native/library-search.png),
[koud heropend](native/library-cold-opened.png).

De oude documentlaag had kleine monospace-invoer in een groot wit veld.
De nieuwe compositie geeft de inhoud, objectnaam en bewerkingscontext een
herkenbare hiërarchie. Alle 28 hostbeelden en 14 nieuwe native librarybeelden
zijn visueel beoordeeld: geen afgekorte kernacties, overlap, technische IDs of
stille previewclipping in de bekeken beelden. De eerste review vond een
ontbrekende en-dash-glyph; 6404431 corrigeert die naar ASCII.

| Niveau | Werkelijk bewijs | Open grens |
|---|---|---|
| Native QEMU | 1280×720, echte invoer, processen, opslag, herstel | Geen fysieke hardware |
| Host | 880×510/100% en 1120×660/200%, actuele echte Rust-renderer | Geen native high-DPI-claim |
| Browser | Geen nieuw browserbewijs nodig voor deze slice | Geen vervanging voor native bewijs |
| Ontwerp | Bestaande volledige sessie-/privacycontracten blijven leidend | Echte persoonlijke authenticatie blijft open |

De compacte 200%-editor verbergt toolbar/metadata responsief; toetsenroutes
blijven werken. De bibliotheek is nog sober. Volledige shaping, IME,
screenreader, verdere native resoluties en menselijke taakproeven blijven open.
De deterministische probe heeft een klokplaceholder; de bestaande openbare
RTC-sessieroute is een afzonderlijk profiel.

## Metingen en resterende traagheid

QEMU 10.0.13, q35-10.0, TCG/thread=single, qemu64, 1 CPU, 256 MiB,
GOP 1280×720, VGA, display none en QMP screendump. Geen NIC, hostschijf of
passthrough; alleen wegwerpdisk en eigen OVMF-vars. KVM is hier niet beschikbaar.
Geen nieuwe Windows/WHPX-, VMware- of hardwaremeting.

| Gemeten write-route | Tijd |
|---|---:|
| Eerste waarneming firmware-exit/kernel-entry | 2,507 s |
| Kernelgeheugen gereed | 3,085 s |
| Vereiste diensten gereed | 8,668 s |
| Eerste complete Materialframe | 76,152 s |
| Complete interactieve bibliotheek | 78,724 s |
| Laatste invoer → bewaard beeld | 1,985 s |
| Geschiedenis openen | 53,349 s |
| Herstellen | 48,198 s |
| Heropenen | 48,505 s |

Dit zijn host-monotone observatiegrenzen met guestwerk, QMP, capture en exacte
pixelcontrole; geen afzonderlijke guest- of scanoutlatency. **De klacht over
45 seconden blijft relevant voor grote overgangen.** U02 blijft open. Een losse
native render-/conversietijd en p95 zijn hier niet geïsoleerd gemeten.

Dezelfde 22 regressiefasen duurden op 703 508,802 s en nu 487,007 s. Enkele runs
op hetzelfde TCG-profiel leveren geen statistische snelheidsclaim op.
QEMU-child CPU user/system: write 381,560/3,400 s, cold 190,930/1,550 s,
regressie 487,760/2,600 s. Fixturegeneratie is hiervan uitgesloten.

[Werkelijke ELF-vergelijking](resource-comparison.json): LOADpagina's
**195 → 156**, 39 pagina's/159744 bytes minder; bestand 634424 → 481544 bytes.
R: 471192 → 291720 bytes; RX: 110809 → 133817 bytes; RW/BSS: 200704 bytes,
ongewijzigd. Vaste adressen en vijf W^X-segmenten blijven geldig. Dit meet
geladen appsegmenten, niet totaal proces-/kernelgeheugen of snelheid.
Het vaste RX-budget is gemotiveerd [128 → 144 KiB](../../implementation/NATIVE-MATERIAL-LIBRARY.md);
R/RW, stack/guards, drie 64-KiB-buffers en kernelquota blijven gelijk.

## Controles, correcties en open punten

[Exacte commando's, exitstatus en tijden](checks/). Op ca503c8 slagen 76 UI-,
41 app-, 109 SDK-, 190 Python-tooltests en 11 runnerchecks, Clippy met
`-D warnings` en formatting. Na de enige glyphcorrectie zijn de 76 UI-tests en
de echte acht ELF's/kernel/UEFI-build opnieuw geslaagd op 6404431. De finale
Materiaal-ELF slaagt met 156 pagina's in dezelfde kernelparser.

Nieuwe negatieven behandelen read-only, dirty, CAS/stale domein/revisie,
onzekere publicatie/timeout, fout na restore-ACK, ongeldig UTF-8 buiten de
previewprefix, stale semantische acties, muisknop over dialoogwisseling,
compactfocus en collectiecreate gevolgd door META-conflict zonder replay.
Een code-/AI-review is geen onafhankelijke menselijke audit.

Een vroege writeproef is expliciet gestopt wegens de glyphfout; een eerdere
verdwenen toolsessie heeft geen eindexit. Beide blijven diagnostiek en tellen
niet als geslaagd. De oude native runner gebruikte bovendien nog de eerdere
editorrechthoek. De [echte Frame-reproductie](legacy-oracle-result.json) vindt
51 legitieme metadatapixelwijzigingen in die oude rechthoek en nul wijzigingen
in de actuele editor. Die toolingcorrectie krijgt eigen behoudsnegatieven;
de hier gebruikte volledige appbeelden blijven ongefilterd.

[Alle 18 CI-workflows op 640 zijn daadwerkelijk groen](ci-640-workflows.json).
[Native CI-run](https://github.com/simonvanieperen/nieuw-os/actions/runs/41527945142)
doorliep full, write en cold en maakte de gewone Windows-QEMU-bundel. De
CI-merge `f6bada8333c63e0a949d8a441b8494cac56b3503` heeft exact dezelfde
source-v3. Artifact 13183010575 is opgehaald en de ZIP-hash bevestigd;
de aanvullende inhoudreadback wordt apart vastgelegd. Dit is geen nieuwe
Windows-runtime-, VMware-, hardware- of volledige releaseclaim.

Ook de [openbare native sessieroute](native-public-route.json) slaagt opnieuw:
18 toestanden, raw 33, 190,776 s. Lockscreen interactief op 7,769 s, complete
desktop op 91,174 s. Echte klok, gesloten persoonlijke methoden, expliciete
openbare VM-gast, zoeken, instellingen, cover en shutdown zijn vastgelegd.
Dit is geen echte persoonlijke authenticatie. [Lockscreen](native/public-lock.png),
[desktop met klok](native/public-desktop.png), [cover](native/public-covered.png).
Echte profiel-/auth-/sessieauthority en capability-intrekking bij persoonlijke
lock/logout blijven P0; dagelijks vloeiende graphics en volledige input/
toegankelijkheid blijven apart open.
