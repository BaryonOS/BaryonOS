# G06 — gerichte review van de centrale integratie

## Oordeel

**Geen blokkerende integratiefout gevonden in de gecontroleerde hooks.**
De SDK-selectie blijft beperkt tot M6c, de quantumproef heeft een apart
M3-profiel en de standaard M3-uitvoering behoudt haar bestaande paden.
De werkelijke gecombineerde bron heeft de volledige SDK-M6c-acceptatie,
volledige M3-regressie en strikte native quantumacceptatie doorstaan.
G06 voert geen nieuwe VM of algemene testsuite uit en telt de bestaande
runs niet opnieuw als nieuwe uitvoering.

Er is één concreet, niet-blokkerend verbeterpunt voor bronbinding gevonden:
het parentmanifest van qsim-core ontbreekt in de globale v3-fingerprint.
De definitieve quantumrunner controleert datzelfde inputbestand afzonderlijk
vóór en na de run. Het huidige quantumresultaat is daardoor wel aan dit
bestand gebonden; de hoofdagent heeft de globale fingerprintaanvulling op
de werkqueue gezet.

## Bron en gecontroleerde overgang

Beoordeeld is `3e41bbf93822f7044c6020259f4a752925e1966d`, met v3-fingerprint
`307c828e9ce7bf49a4e809859ca77ecd332015572dd37e96fb580de98951aa60`.
De relevante diff is gelezen ten opzichte van uitgangsbron
`1acf6e8cde2a63849dc49c948ad97ba7f2c39cad`. `review.json` bindt de gelezen
bestanden aan de exacte commitbytes en de gebruikte bestaande bewijsbestanden
aan hun SHA-256.

De review omvat `tools/build.py`, `tools/build_users.py`, `tools/common.py`
en de nieuwe hooks in `kernel/src/m2/m3.rs`. Alleen direct benodigde
configuratie en input-/acceptatiecontrole zijn aanvullend gelezen:
kernel-features, `kernel/build.rs`, `.cargo/config.toml`, de quantum-manifests
en `run_qsim.validate_build`. SDK-internals, de complete kernel en andere
OS-onderdelen zijn niet opnieuw algemeen geaudit.

## Gecontroleerde integratiegrenzen

| Grens | Codebeoordeling en toepasselijk bestaand bewijs |
|---|---|
| SDK uitsluitend in M6c | `NOS_SDK_RUNTIME` accepteert exact `0` of `1`; alleen een gekozen M6c-kernelfeature kan `use_sdk=true` maken. `build_users(..., sdk_runtime=true)` weigert ieder ander fixturetype. De rlib en `nos_sdk_runtime`-cfg worden alleen bij het M6c-programma toegevoegd. G05 controleert alle 30 echte M6c-kernels op `sdk_runtime=true` én de ingebedde SDK-user-ELF; de aansluitende gewone M1-run is afzonderlijk. |
| Default blijft default | Zonder expliciete SDK-selectie blijft het originele M6c-runtimepad actief. De nieuwe `native-qsim`-cfg wijzigt uitsluitend zijn eigen ELF-selectie, procesaantal, startupwaarde, manifest, schedulerassertie en observer. Met de cfg uit blijven de oude M3-expressies en observerpaden gelijk. De volledige M3-regressie controleert dit gedrag op dezelfde gecombineerde bron. |
| Quantumprofiel exclusief | `native-qsim` vereist M3. Combinatie met M4 of G0 levert een compile error; M5/M6 erven M4 en vallen daardoor onder dezelfde grens. De centrale build bouwt het quantumartifact uitsluitend voor exact `native-qsim`; die profielnaam matcht de afzonderlijke gewone m3/m4/m5/m6-buildtak niet en overschrijft het gekozen `NOS_QSIM_ELF` dus niet. |
| Artifact werkelijk geladen | De nieuwe `NOS_QSIM_ELF`-invoer heeft zowel env- als bestandswijzigingstracking in `kernel/build.rs`; de bestaande M6c-invoer behoudt die tracking. De quantumrunner controleert scalar-rapport, actuele broninputs, ELF, disassembly, loader/kernelhash en de volledige aanwezigheid van die gebruikers-ELF in de gekozen kernel. G05 controleert de echte SDK-ELF op dezelfde manier aanvullend voor alle M6c-kernels. |
| Bronbinding van nieuwe onderdelen | De globale fingerprint omvat SDK, native quantumbron en qsim-core, naast de bestaande kernel/dienst/toolinputs. De ontbrekende geërfde parentmanifestinvoer staat hieronder als concrete uitzondering. De quantumrunner vergelijkt voor en na uitvoering de volledige inputhashmap van de native builder, inclusief deze uitzondering. |
| Gescheiden uitvoer | `BUILD` en `TARGET` volgen expliciete `NOS_BUILD_DIR`/`NOS_TARGET_DIR`. Beide kernel-/loader-Cargobuilds gebruiken die targetmap; alle user-ELF's en SDK-rlib staan onder die buildmap; quantum krijgt bovendien `TARGET/qsim-user`. De bestaande eindruns gebruiken drie verschillende buildmappen en targetmappen, en stellen ook `CARGO_TARGET_DIR` voor host-Cargo in. |
| Standaardprofielen en QEMU | De standaardpaden blijven `ROOT/build` en `ROOT/target` wanneer geen isolatie is gekozen. QEMU-machinekeuze gebruikt een expliciete beperkte lijst; er is geen stille fallback tussen versies. De beoordeelde runs leggen het gekozen 7.2-profiel en hun artifacts vast. |

De exclusiviteitsbeoordeling van de featureguards is bronreview; G06 presenteert
geen niet-uitgevoerde extra compile-fail-test. De runtimekant is gedragen door
de opgeslagen native uitvoeringen hieronder.

## Bestaand gecombineerd bewijs

| Bundel | Uitvoering op de beoordeelde bron |
|---|---|
| `parallel-final-sdk-m6c` | G05: 35 groepen, 31 echte VM's, waaronder 30 SDK-M6c-runs, 14 koude herstelparen en één M1-regressie. Capture en aanvullende SDK-artifactbinding geslaagd. |
| `parallel-final-m3` | I05: 22 groepen, waarvan drie hostgroepen en 19 echte VM's. De 18 M3-images controleren onder meer private adresruimtes, preëmptie/fairness, resourceopruiming, exit/idle, grote ELF, foutpointers, guardpages en x87/XMM/AVX-afwijzing; daarna een gewone M1-regressie. |
| `parallel-final-qsim` | I05: één echte VM met twee ring-3-processen, exacte referentie-uitkomsten, geïnjecteerde afbreking, echte core-geheugenuitputting en 96 vrijgegeven frames. De strikte runner bindt de scalar-ELF en alle buildinputs vóór en na uitvoering. |

Dit zijn samen 51 reeds uitgevoerde, afzonderlijke VM-invocations uit drie
bestaande bundels, **nul nieuwe VM-invocations door G06**. Ze gebruiken dezelfde
uitvoeringscommit en v3-fingerprint. De opname van reviewdocumentatie verandert
die uitvoeringsbron niet.

## G06-01 — ontbrekende geërfde manifestinvoer in globale fingerprint

`experiments/quantum/qsim-core/Cargo.toml` erft `version`, `edition`,
`rust-version` en `publish` uit `experiments/quantum/Cargo.toml`.
`common.source_hash()` doorloopt qsim-core en native, maar niet dit
bovenliggende manifest. De gerichte reproductie bouwt uitsluitend in een
`TemporaryDirectory` een kleine fixture en verandert daar `edition = "2021"`
in `edition = "2024"`. De manifestbytes veranderen; de berekende globale
fingerprint blijft gelijk. De echte projectbron is niet gewijzigd.

Dit is geen blokkerende fout voor de huidige native quantumacceptatie:
`native/build.py::sources()` neemt het parentmanifest op en
`run_qsim.validate_build()` eist dat de opgeslagen inputhashmap exact
overeenkomt met die actuele functie. De runner doet deze controle zowel vóór
als na uitvoering. De finale quantum-bouwrapporthash voor het parentmanifest
komt bovendien overeen met de actuele bevroren bron.

Kleinste verbetering op de queue: laat de globale fingerprint het
parentmanifest meenemen en voeg een gerichte controle toe die aantoont dat
een wijziging daarvan de fingerprint wijzigt. Eigenaar: hoofdagent voor
`tools/common.py` en de passende hosttest. Een historische capture wordt niet
naar een nieuwe fingerprint omgelabeld; haar bestaande eigen bron- en
artifactbindingen blijven bewaard. De volledige suites hoeven voor deze
review niet opnieuw te worden gestart.

Reproductie van de beperkte bevinding:

```sh
python3 docs/verification/parallel-final-integration-review/fingerprint-omission.py /pad/naar/nieuw-os
```

`fingerprint-omission.json` bevat de uitgevoerde uitkomst en de bronhash van
de gebruikte fingerprintcode. Deze reproductie controleert geen wijziging
aan de compiler of gast; zij toont uitsluitend de gemiste input aan.

## Afbakening en bijkomende vraag

Dit is een gerichte AI-review, geen menselijke audit. De eerder gemelde
G0-serialwijziging valt onder de afzonderlijke forensics van de hoofdagent.
Op diens verzoek zijn alleen de eigen recente scripts en M6c-fixturewrites
nagekeken: die schrijven naar eigen SDK-build/evidencepaden of tijdelijke
mappen; geen write naar het oorspronkelijke G0-pad is gevonden. Deze agent
heeft die G0-run niet hersteld of overschreven. Dat bepaalt niet de oorzaak
van de elders waargenomen wijziging.

De review wijzigt geen bron, interface, feature, acceptatiecriterium of
roadmapgewicht. **Roadmapdelta 0 punten.** De nieuwe SDK- en quantumresultaten
behouden hun in de afzonderlijke bundels omschreven grenzen.
