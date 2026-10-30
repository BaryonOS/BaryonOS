# INTERACTIVE-COLD-1: codecheckpoint, native acceptatie nog open

Afzonderlijke branch `simon/native-cold-acceptance`, basis
`3d2e1aba218feb39e02a1a5d257c5eb63f50e26b`. Geen onderdeel van de bevroren
PR #66-reviewbron. Nieuwe files `tools/materiaal_cold.py`,
`tools/run_materiaal_cold.py`, `tools/tests/test_materiaal_cold.py`.
Geen bestaande preview/probe/kernel/SDK/manifest gewijzigd.

Doel: een normale zevenprocessen-save/reopen-parent strikt opnieuw
valideren (bron, kernel/loader, alle appbindings, QMP-inputs, fasebeelden,
canonical empty before en onafhankelijk gedecodeerde actieve afterdisk).
Daarna een verse normale VM op een bytekopie van die disk starten.
Het koude lijstbeeld wordt volledig verwacht: populated-listpixels plus
de exacte Ready-statuspixels volgens `Layout::status`; geen maskers,
geen ad-hoc vergelijking die een gebied negeert. Heropenen vereist het
volledige oorspronkelijke native heropenframe, daarna QMP-quit/raw0,
onveranderde diskbytes en opnieuw parent/reference/source-emulatorcontrole.

**Geen guest clean shutdown:** de gewone variant heeft geen geaccepteerd
OS-sessieafsluitpad. Parent en child stoppen via expliciete host-QMP-quit.
Dit bewijst dus geen cleanup, nul-I/O-counters, writeweigering, versleutelde
productopslag of volledige gebruikerssessie/recovery. Release-eigenaar is
van deze ontbrekende runtimefunctionaliteit op de hoogte.

Uitgevoerd:

```sh
python3 -B -m unittest tools.tests.test_materiaal_cold
```

Zes hosttests, exit0; log `/tmp/nieuw-os-cold-helper-tests.log`.
Negatieven: bron/kernel/machine/raw-exittype, getamperde disk en frame,
QMP-input/response/index/quit, oude successreport bij prep-fout.
Hostfixtures zijn geen native bewijs.
De kandidaat-helper is eerder buiten de bron op historische echte artifacts
geprobeerd; volledige parentreplay en vooraf renderer-afgeleid koud lijstframe
slagen, maar die verkenning is geen nieuwe bronacceptatie.

**Niet uitgevoerd:** volledige toolsuite en daadwerkelijke native gateketen
op deze nieuw gewijzigde v3; runner-lifecycle/failuretests moeten nog verder
worden aangevuld vóór acceptatie. Werk tijdelijk gepauzeerd om verplichte
actuele-mainintegratie van PR #62 uit te voeren op de aparte integratiebranch.
Hier niet ongemerkt een tweede actieve roadmap of groen coldcriterium
van maken. Centrale status/queue/assessment zijn van de teamleider.
Voortgangvoorstel **34% → 34%**.
