# M6a — uitgevoerd bewijs en observerherstel

28 oktober 2026. Runtimeherstel `f60445b`, geïntegreerde basis `803bf43`;
fingerprint v3 `e721bec4c283c90fe6d05a66d092372cd59f7727815de79f0da18760155c365c`.
Contract vóór implementatie: docs/implementation/M6A-WRITE-RECOVERY.md.

| Bestand/map | Betekenis |
|---|---|
| suite.json,guest-runs.json | volledige M6a-poort:54 groepen/49 echte VM,exit0 |
| observer-counterexamples.json |2 oude false-greens +6 nieuwe afwijzingen,8 echte VM,helper exit0 |
| regression/{m1..m5}/ |77 groepen/64 VM,alle suites/captures exit0 |
| manual-m6a.json,final-build-manifest.json | normale CLI-eindstart,raw33,gen2 |
| observer-review.md | concrete bronreview,geen blokkerende bevindingen |
| verifier-m6a.json,verifier-regression.json | afzonderlijke AI-controle van fysieke rawlogs/artifacts/journalbytes |
| validation.json | exacte opdrachten,exitstatus,bronpin,gewijzigde bestanden,herstelgrenzen |
| history/6fce5c12/ | eerdere54/49-poort vóór de strengere observer;geen actueel eindbewijs |
| diagnostics/,observer-diagnostics/ | historische en afgewezen pogingen;geen groen eindbewijs |

131 gewone controlegroepen/113 VM plus8 tegenproeven en1 normale eindstart:
122 VM in het gezamenlijke eindbewijs. Negatieve guestuitkomsten zijn niet
naar succes herschreven. Bij de6 verwachte observerafwijzingen blijft de
ruwe run passed:false/raw35; de helper bevestigt de bedoelde assertion.

De eerste geneste tegenproefworktree erfde Cargo-rustflags tweemaal en stopte
vóór de observer. Een daaropvolgende oude extra-FLUSH-proef werd al door een
oude assertion afgewezen; de verwachting van een oude false-green was daar
onjuist. Beide zijn afgewezen diagnostiek. Alleen de definitieve externe
worktree met exact één kernellinkerscript levert de8 eindtegenproeven.

Rawhashes verwijzen naar behouden lokale bestanden; Git bevat compacte
tekst/JSON en hashes, geen binaries/images/caches/secrets. Geen fysieke
stroomuitval- of hardwareclaim. Een nieuwe checkout kan tests reproduceren,
maar reconstrueert niet de oude willekeurige images. FNV is diagnostiek,
geen cryptografische authenticatie. De logs bevatten geaggregeerde
observerresultaten,geen volledige per-IPC/ACK/I/O-eventtrace.
