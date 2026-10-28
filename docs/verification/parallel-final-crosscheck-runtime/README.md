# I06 — onafhankelijke controle van het definitieve M6b-bewijs

## Uitkomst

**Geen concrete blokkade gevonden.** Deze gerichte controle leest de echte
bewaarde schijfbestanden, kernel-/loader-/firmware-artifacts en ruwe logs
van B05. Zij start **geen nieuwe VM** en wijzigt geen schijf of broncode.
De 28 hieronder genoemde runs zijn bestaande B05-runs; zij mogen niet
nogmaals bij het aantal uitgevoerde native tests worden opgeteld.

Broncommit: `3e41bbf93822f7044c6020259f4a752925e1966d`.
Source-fingerprint v3:
`307c828e9ce7bf49a4e809859ca77ecd332015572dd37e96fb580de98951aa60`.
HEAD en bronhash zijn voor en na de crosscheck gelijk.

| Gecontroleerde grens | Rechtstreekse waarneming |
|---|---|
| Exacte native dekking | 28 unieke VM-mappen met 28 fysiek verschillende seriële logbestanden; 26 rawstatus 33 en twee echte hostkills met rawstatus −9 |
| Gearchiveerd versus oorspronkelijk | Iedere opgenomen gast-JSON is exact gelijk aan het oorspronkelijke runresultaat; de seriële/stderrbytes komen overeen |
| Artifacts | Alle opgenomen artifacthashes opnieuw berekend over de daadwerkelijke bestanden, inclusief schijf-voor/na, kernel, loader, firmware en manifests |
| Volume-identiteit | 14 afzonderlijke volumes; het actuele schijfbestand is exact gelijk aan de bewaarde nasnapshot |
| Beginbeeld en canaries | Beginbeeld opnieuw opgebouwd uit de vastgelegde seed; bytes vóór en na het journal zijn onveranderd |
| Koud herstel | 13 afzonderlijke vervolg-VM's: vorige nasnapshot = volgende voorsnapshot = volgende nasnapshot, byte voor byte; telkens nul mutation-ACK's |
| Published-WRITE-grens | `write-timeout`, `reset-timeout` en `write-poll-limit` hebben hier alle drie sequence 0 en exact het beginbeeld |
| Foutuitkomst | Alle zeven foutcases hebben exact één `Unknown`, geen ACK-/objectuitvoer en de vereiste no-replay/no-stale-read-markers |
| Reset-time-out | Exacte reeks reset-request → masked ACK → quarantine → pinned pool; geen poolhergebruikmarker |
| Twee canarybeelden | De normale A/B-runs gebruiken verschillende beginbeelden met dezelfde kernel en dezelfde gecontroleerde documenttoestand |

## Onafhankelijke werkwijze

`check_m6b.py` gebruikt de M6b-runner, capture-validator en disk-oracle niet
als beslisser. Het herberekent SHA256 rechtstreeks, reconstrueert de
canaries en leest de journalheaders, padding, bestaande FNV-checksums,
sequence, request-ID, object, revisie, operation en payload rechtstreeks
uit de sectorbytes. De berekende toestand wordt vergeleken met de ruwe
native toestandregels en het opgeslagen resultaat. FNV is hier alleen de
bestaande corruptiecontrole, geen authenticiteitsgarantie.

Voor de drie gepubliceerde maar onbevestigde WRITE-gevallen accepteert de
controle uitsluitend het exacte lege beginbeeld of één canoniek eerste
record: sequence 1, request-ID 10, object 0, revisie 1, WRITE en de exacte
fixturepayload. **De daadwerkelijke native waarneming is in alle drie
gevallen sequence 0.** Sequence 1 blijft de begrensde toegestane andere
uitkomst; deze crosscheck claimt niet dat die hier is waargenomen.

Bij herstel worden niet alleen hashes of `passed: true` vergeleken. De drie
betrokken bytebuffers moeten exact gelijk zijn en dezelfde volume-identiteit
behouden. Daarmee is in deze runs ook daadwerkelijk geen replay-write te
zien. De faultmarkers worden als volledige geordende M6B-regelreeks
vergeleken, inclusief reset en poolbeleid.

## Reproduceren en bewijs

```bash
python3 -B docs/verification/parallel-final-crosscheck-runtime/check_m6b.py
```

De bewaarde B05-artifacts onder `build/parallel-final-m6b/run` moeten aanwezig
zijn. `result.json` bevat de directe hashes, journaluitkomsten en alle
13 herstelkoppelingen. `execution.json` en `stdout.txt` bewaren de werkelijk
uitgevoerde read-only controle; `artifact-index.json` bindt de bewijsbestanden.

Deze controle sluit de gerichte B05-claims op deze bron aan op hun bewaarde
native artifacts. Zij voegt geen hardwarebewijs, nieuwe OS-functionaliteit
of roadmapscore toe. De oorspronkelijke B05-suite en de onafhankelijke
I05-quantum-/M3-runs blijven afzonderlijk gelabeld.
