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

## Hervatting TL-BUILD-COLD, 30 oktober 2026

Actuele integratie `c0962e00a1ce6a47bd952c9fea99d437b327e4d4` is uitsluitend
in deze aparte branch opgenomen (mergebasis `cd5f35fce7ef97d84f91b2f728c06d7d8e53ba79`).
PR #66 blijft op c096 bevroren. Alleen de drie toegewezen cold-toolbestanden
en eigen bewijs zijn gewijzigd; bestaande preview/reference-orakels,
kernel, SDK, workflows en centrale voortgang blijven ongemoeid.

Nieuwe host-lifecycletests controleren post-quit/raw0-diskinspectie,
ongeldige parent zonder VM-start, nonzero exit, gewijzigde disk/source/
executable, afwijkende QEMU-implementatie en timeout met eigen procescleanup.
De coldtrace wordt zelfstandig tegen de volledige vaste invoervolgorde
herhaald; bron, parent, referentie en gekopieerde onveranderlijke artifacts
worden na exit opnieuw gecontroleerd. Mutable OVMF-vars zijn daarvan uitgezonderd.

Uitgevoerd, alle exit0:

```sh
python3 -B -m unittest tools.tests.test_materiaal_cold
python3 -B -m unittest discover -s tools/tests
python3 -B tools/run_materiaal_cold.py --help
git diff --check
```

14 gerichte hosttests en 152 tools-tests groen.
Logs `/tmp/nieuw-os-cold-lifecycle-tests.log` en
`/tmp/nieuw-os-cold-all-tools.log`; de eerdere hosttestmock die bij terminate
onterecht QMP-quit vereiste is hersteld, geen productieassertie versoepeld.

**Native keten nog niet uitgevoerd op deze nieuwe v3:** eerst broncommit
bevriezen, daarna nieuwe reference + gewone parent + cold-child met eigen
`target/native-cold-acceptance/gate1` en `build/native-cold-acceptance/gate1-*`.
Dit toetst herstel na expliciete geforceerde host-QMP-stop na opslaan,
geen guest clean shutdown en geen abrupte stroomuitval tijdens een write.
Hier niet ongemerkt een tweede actieve roadmap of groen coldcriterium
van maken. Centrale status/queue/assessment zijn van de teamleider.
Voortgangvoorstel **34% → 34%**.
