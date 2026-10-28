# Actuele projectstatus — M6a

28 oktober 2026. M6a en de drie aangescherpte testcontroles zijn uitgevoerd en
in QEMU geverifieerd op branch `codex/m6a-write-recovery`. Oorspronkelijke PR
https://github.com/simonvanieperen/nieuw-os/pull/5 is al gemerged op `803bf43`;
dit latere observerherstel staat als nieuwe commits op de branch. Basis `803bf43`;
runtimeherstel `f60445b26d31733cc6aaf557ee08820b2295b0df`.
Bronfingerprint v3:
`e721bec4c283c90fe6d05a66d092372cd59f7727815de79f0da18760155c365c`.
Latere commits bewaren testscripts, documentatie en bewijs.

## Wat werkt

- Eigen UEFI-loader/no_std-kernel, procesisolatie, capabilities en IPC blijven
  werken. De eigen ring-3-blokdienst schrijft op een synthetische 64KiB-image.
- Alleen WRITE-completion gevolgd door echte virtio-FLUSH-completion geeft
  een app-ACK. Vier journalrecords bewaren één waarde van 128 bytes. Vol geeft
  ENOSPC vóór I/O; onzekere afloop geeft Unknown zonder automatische replay.
- FULL vereist de vierde PUT én ontvangen ENOSPC-reply. Alle vijftien
  malformedgevallen vereisen hun eigen inhoud en unieke request/send/receive-
  dekking. Unknown vereist terminale replyontvangst, nul ACKs, exacte WRITE/
  FLUSH-aantallen en geen volgende muterende syscall/publicatie.
- Zes echte dienstcrashes en zes externe QEMU-proceskills zijn gevolgd door
  de toepasselijke recovery. Zeventien koude herstelparen behouden exact
  dezelfde imagebytes, identiteit en runkoppeling.

## Werkelijk uitgevoerd

| Opdracht | Resultaat | Bewijs onder verification/m6a/ |
|---|---|---|
| `python -X utf8 tools/preflight.py --strict` | exit0 | m6a-preflight.* |
| Python unittest discovery onder tests/ | exit0,43 controles | validation.json,host-regression.* |
| `python -u -X utf8 tools/test_m6.py` | exit0,54 groepen/49 VM | suite.json,guest-runs.json |
| `python -u -B -X utf8 tests/verify_m6_observers.py` | exit0,8 VM | observer-counterexamples.json |
| `python -u -X utf8 tools/test_milestones.py --through m5` | exit0,M2–M5:67 groepen/56 VM | regression/{m2,m3,m4,m5}/ |
| `python -u -X utf8 tools/test.py` | exit0,M1:10 groepen/8 VM | regression/m1/ |
| Zes `capture_gate.py`-opdrachten, M1–M6a | exit0,raw/hash/coverage gecontroleerd | genoemde archieven |
| Normale M6a-build + `python -u -X utf8 tools/run_m6.py --smoke` | exit0,raw33,gen2 | manual-m6a.json,final-build-manifest.json |

M1–M6a tellen samen 131 controlegroepen/113 VM; daarnaast acht tegenproeven
en één normale eindstart, dus 122 afzonderlijke VM-runs in het eindbewijs.
De M6a-hostgroepen bevatten 87 Rust-tests,20 harnascontroles en12 ELF-controles.
M1-negatieve runs blijven `passed:false`, inclusief de echte 60s-time-out.

Twee afwijkende clients werden door de oude observer nog geaccepteerd. De
nieuwe observer wijst die en vier Unknown-tegenproeven op de exacte assertion
af (raw35,geen PASS). Drie tegenproeven wijzigen echte guestcode; drie testen
observerinstrumentatie en bewijzen geen extra werkelijk device-I/O.

Review: observer-review.md. Onafhankelijke raw/hash/journalaudit: verifier-m6a.json;
afzonderlijke gecombineerde regressieaudit: verifier-regression.json. Beide
exit0,nul fouten:122 unieke VM,677 artifacthashes en alle raw serial/stderrparen.
Dit zijn AI-controles, geen menselijke audit. Exacte opdrachten, wijzigingen,
logs, bronpin en herstelgrenzen: validation.json en README.md in de bewijsmap.

## Starten en grenzen

```powershell
python tools/build.py --kernel-feature milestone-m6a
python tools/run_m6.py --smoke
```

De eindige seriële proef sluit QEMU na de assertions af. Er is hier geen
algemeen bestandssysteem, GC, native desktop of fysieke hardware/QPU-test.
QEMU-proceskill bewijst geen fysieke stroomuitval; hostcaches blijven bestaan.
Kerneltransport en protocolconform QEMU/backend zijn vertrouwd; geen IOMMU-
of kwaadaardige-DMA-containmentclaim. Contract: implementation/M6A-WRITE-RECOVERY.md.
Materiaal en qsim blijven afzonderlijke hostprototypes; deze ronde herhaalt
hun UI-/quantumtests niet. Geen hostdisk, NAS, firmware of productiekeys gebruikt.

Ouder M6a-bewijs blijft onder history/6fce5c12/: de oude54/49-poort was groen,
maar de aansluitende regressie stopte bij een Windows-filelock. De huidige
regressie is volledig herhaald. Afgewezen tegenproefopstellingen blijven onder
observer-diagnostics/ herkenbaar; een geneste Cargo-config verdubbelde daar
het linkerscript. De definitieve helper gebruikt een externe wegwerpworktree.

De Codex-routingfout is afzonderlijk onderzocht en privé geback-upt. Een
permanent herstel is in deze chat niet aangetoond. De aparte actieve chat
'Herstel routing-time-outs in Codex' voert appherstel uit; deze werkronde
wijzigt die configuratie niet tegelijk. Appdiagnose is geen OS-testbewijs.

Vervolg: integreer dit observerherstel met de afzonderlijke vervolgbranch en
voer daar de gecombineerde poorten uit. Andere worktrees en bestaande
`.codex/environments/` zijn behouden. Geen merge/release uitgevoerd door deze
werkronde. Bronherstel via aparte checkout of gerichte revert, nooit door
onzekere writes opnieuw te spelen of een bestaand volume te formatteren.
