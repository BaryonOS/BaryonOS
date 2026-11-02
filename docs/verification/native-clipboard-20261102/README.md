# Native klembord — brongebonden deelacceptatie, 2 november 2026

**Geslaagd:** Ctrl+C/Ctrl+V tussen twee geïsoleerde native apps, uitsluitend
voor het eigen nog aangeboden invoergebeurtenis en actieve venster. Eén gebaar
autoriseert één verzoek; Ctrl+Shift+Delete wist het vluchtige klembord.
Een compositorherstart begint leeg, met behoud van document en berekening.
Dit is een afgeronde klembordstap, geen volledige U02- of releaseacceptatie.
De roadmap blijft **41/100**; interactiebudgetten, ruimere capaciteit en echte
accountauthenticatie blijven open.

## Uitvoering en bron

| Uitvoering | Commit | Source-v3 |
|---|---|---|
| Root, volledige native route | `d0ae570bd4666626096f92e57acef981c0f0d7b6` | `05307fef9684d97480a77ea39050d80f6affcf522a6c09c7eb53340c789b35aa` |
| Finale testbron/CI-trigger | `e3182f0bba72f300596dd81417fbbd371a8d5d79` | `038615b1b425d3f24ea2d9c52b36184ea666f81237c813303dc0fac93fb756ca` |
| Werkelijke CI-merge | `5754763ffb257d6420f74126cae7252e36afab1e` | `038615b1b425d3f24ea2d9c52b36184ea666f81237c813303dc0fac93fb756ca` |

Na d0ae wijzigen uitsluitend beide app-`tests/chain.rs`-bestanden. Root heeft
het probeprofiel op de finale bron opnieuw gebouwd en alle tien binaire
SHA256-waarden gelijk bevonden: loader, kernel en acht programma's.
[Test-only-brug](test-only-binary-bridge.json). Het oorspronkelijke native
rapport behoudt d0ae/source0530; het wordt niet omgelabeld.

De oorspronkelijke dertien fasen blijven intact; negen klembordfasen zijn
bijgevoegd. Vier echte Copy-chords, vijf Paste-chords en één Clear-chord
bewijzen kopiëren in beide richtingen, het atomair weigeren van letters in
Rekenmachine, wissen zonder documentwijziging, opnieuw bevolken vóór een
werkelijke compositorfault en leegte na herstel. De normale tekst wordt vóór
bewaren teruggezet; de onafhankelijke diskoracle vindt object0/revisie1,
veertien bytes, inhoud-SHA256
`9a45a00fa9ff69c17e124f54c14bb14cf2c7650e8cf1c272f6631f618a19a5a5`,
met intacte canaries. Werkelijke resources na afloop: nul processen en handles,
`WINDOW:CLEANUP:PASS`, `WINDOW:CASE:PASS`, raw exit33.

Root-QEMU10.0.13/TCG/q35-8.2/qemu64/één CPU/256MiB/1280×720:
**1196,886763 s**, alle22fasen. Geen netwerk, persoonlijke hostmap of fysieke
doorgifte. Alle opgeslagen fasebeelden en de definitieve serialhash zijn
teruggelezen. Zie [rootresultaat](root-native-verification.json), de originele
`root-d0ae570-native.json.gz`, `root-d0ae570-serial.txt.gz` en buildmanifest.
Het actuele losse volume heeft SHA256
`246c8f9773533796718a729389c26d6103f45f04e7a83b4d06503e17c6d4e07e`.

Alle **18 CI-workflows** op e318 slagen:
[workflowoverzicht](ci-e318-workflows.json),
[werkelijke head per run](ci-e318-workflow-heads.json),
[native windowrun41503460976](https://github.com/simonvanieperen/nieuw-os/actions/runs/41503460976).
De CI-venstergate slaagt in **1407,011261 s**, raw33 en clipboard_gate=true.
De controle van het opgeslagen artifact bindt alle22fasebeelden en34 echte
host-renderorakels aan deze bron. Zeventien volledige appbodyvergelijkingen,
herstel/modelbehoud, firmware/QEMU/ELFs, diskoracle en cleanup zijn gecontroleerd.
De normale bundel wordt in CI pas na de volledige geslaagde gate gebouwd.
Het aparte CI-readbackrapport en de ruwe originelen worden ernaast bewaard.

Dit zijn twee uitgevoerde Linux/TCG-routes; losse duurtijden zijn geen algemene
benchmark. Nieuwe Windows/WHPX- of fysieke runtimeacceptatie is niet geclaimd.

## Rechten, fouten en grenzen

CLP1 loopt over de bestaande inputpair. InputSession blijft de enige RX-eigenaar;
geen tweede eigenaar, kernel-ABI, proces of apparaat-/opslag-/netwerkrecht.
De broker controleert kernel-PID, actuele focus, inputepoch, aangeboden
sequence/generation en echte CtrlC/V-Press. Een appgeschreven intentlabel
verleent niets; een aanbod wordt éénmaal geclaimd. Een uitgestelde response
bewaart uitsluitend een ticket en controleert de autoriteit bij iedere flush.

Een getypeerde Text is begrensd, niet Clone/Debug en wordt bij Drop met de
bestaande gepinde zeroize gewist. Paste wordt pas na oorspronkelijke
ACK Acknowledged toegepast; Superseded of transportfalen verandert de tekst
niet. Reeds succesvol naar de app geënqueuede data kan niet worden teruggehaald.
App-adapters sturen alleen hun actuele selectie; de broker bewijst de
invoergebeurtenis en de focus, geen inhoudsattestatie voor willekeurige apps.

De eerste versie ondersteunt192 UTF-8-bytes in één bericht, zonder afkappen,
historie, hele-document- of oude-antwoordfallback. Niet-tekstformaten, knippen,
bredere capaciteit en algemene app-/sessieintegratie blijven open. De
CtrlShiftDelete-latch kan bij verloren DeleteRelease conservatief één volgende
Delete-cyclus inslikken; zij voorkomt een vertraagde Delete-herhaling.

28 client- en40 brokerhosttests, scalar/native builds en strikte Clippy/fmt
slagen. De volledige UI/apphosttests zijn uitgevoerd. De46 gerichte appketentests
gebruiken de werkelijke capability-engine, INP-service, SUR-client en een kleine
CLP-testtegenpartij; zij dekken opt-in/default-uit, echte focusannulering en
transportverlies vóór én na server-ACK, zonder edit/evaluatie/valse appACK.
Dat is hostbewijs, geen vervanging van de native brokerroute.
[Appnegatieven](app-transport-negatives.json),
[componentlogs](broker-client-component-logs.json),
[preflight](preflight-index.json),
[scoped AI-review](integration-review.json). Geen menselijke audit.
187 Python-tooltests zijn uitgevoerd:184 geslaagd,3 expliciet overgeslagen;
16 gerichte runner/latencychecks slagen (waarvan zes ook in die187 staan).

De fasebeelden vergelijken ieder pixel van de volledige appbody, zonder interne
maskers. De meegerapporteerde READY/GEN-snapshot kan achterlopen op het al juiste
beeld. Dit bewijst volledige appbodypixels plus echte inputACKs, geen causale
laatste-generatie/alle-SUR-ACK-binding. De tijdvelden zijn conservatieve
host/QMP/observatie/capture-intervallen voor appbeelden, geen whole-screen-,
gast-, scanout- of p95-acceptatie. tools/window_latency.py houdt minimaal20samples
per vooraf vastgelegd stratum en alle fouten bij; die volledige workload is
nog niet uitgevoerd. De TCG-schermopbouw blijft te traag voor dagelijkse bediening.

Exacte commando's en hashes staan in de rapporten; uitvoering:

```sh
source /workspace/.nieuw-os-env/activate.sh
NOS_BUILD_DIR=/workspace/nieuw-os/build/clipboard-native \
NOS_TARGET_DIR=/workspace/nieuw-os/target/window-final \
python3 -B tools/build.py --kernel-feature native-window-session-probe
NOS_BUILD_DIR=/workspace/nieuw-os/build/clipboard-native \
NOS_TARGET_DIR=/workspace/nieuw-os/target/window-final NOS_QEMU_MACHINE=pc-q35-8.2 \
python3 -B -u tools/run_window_session.py --clipboard --timeout 1800 \
  --bios-rom /workspace/.nieuw-os-env/sysroot/usr/share/seabios/bios-256k.bin \
  --vga-rom /workspace/.nieuw-os-env/sysroot/usr/share/seabios/vgabios-stdvga.bin
```

Geen algemene veiligheidsvergelijking, echte aanmelding/FIDO2, productie-release
of fysieke-hardwareacceptatie. De volledige releaseopdracht blijft behouden.

## Windows-testpakket

De normale CI-bundel bevat geen crashinjectiecontroller. Alle19payloads, acht
werkelijk uit de kernel teruggelezen app-ELFs, zeven fontinputs en firmware zijn
gecontroleerd. Linux-PowerShell fakeCheckOnly slaagt zonder VM of state; de
20gerichte Python-,22launcher- en33helperchecks zijn eenmaal uitgevoerd.
Dit bewijst geen nieuwe Windows/SDL/WHPX-runtime van het normale pakket.

```text
/workspace/nieuw-os-windows-package/build/clipboard-e318/NieuwOS-klembord-e318-Windows-QEMU.zip
SHA256 e29a6ca5933a0958b10238ae2f660237bbf5295c170d5c92146eb9d4ddf2fc3d
```

[CI-readback](ci-e318-verification.json), [archiefindex](ci-e318-archive-index.json)
en de ongewijzigde gecomprimeerde CI-native-/serialoriginelen staan ernaast.
De apart lokaal gebouwde kandidaat heeft een andere ZIP/kernel/loaderhash en
blijft afzonderlijk vastgelegd; geen binaire-identiteitsclaim tussen CI en die
andere bouwomgeving. Alleen de root-probebuild vóór/na de twee testwijzigingen
heeft de expliciete tien-binaries-brug. Een tweede AI heeft het CI-artifact
onafhankelijk gecontroleerd; geen menselijke audit. Alle18CI-workflows zijn
groen op de genoemde bron. De hoofdbranch wordt door de bestaande integrator
beheerd; deze stap wijzigt geen fysieke machine- of beveiligingsinstelling.

## Integratie en hervatten

PR124 is na de groene broncontrole in de desktopbranch van PR122 opgenomen als
9863a52651504c4838d6826b3e1b2240ae0ed6b2. Geen merge naar main. Code en
bewijs blijven gescheiden van nog open U02-interactiebudgetten. De volgende
werkstap is de trage volledige hertekenketen meten en met behoud van alle
SUR1-/IPC-grenzen verbeteren; geen veiligheidscheck uitschakelen voor snelheid.

De originele agentfinalchecks verwijzen met snapshot_sha256 naar
ci-e318-agent-workflows-original.json; roots eigen genormaliseerde API-read heeft
een andere wrapper/timestamp en een eigen SHA. [Snapshotbindingen](ci-snapshot-bindings.json)
bewaren beide afzonderlijk en koppelen ieder van de achttien werkelijke runheads
opnieuw aan e318 via de echte GitHub-runmetadata. Geen origineel omgehasht.
