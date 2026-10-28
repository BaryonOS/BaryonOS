# M6d-reviewherstel en aanvullende Windowsacceptatie

28 oktober 2026. Basis `7484d87ec639fd8a29be327cad47c2f97739199d`;
geteste runtimecheckpoint `326d2f6efa20411d5bb349b8275bc3e741be7ae0`.
Fingerprint v3 `5c69802a6de7c591207950f311d2e9e4f7eb07628d3b91dc9457f21c0bfa8c75`.
Native kernel/service/probe zijn bytegelijk aan oorspronkelijke bron 88e6c04.
De runtime-toolwijziging is uitsluitend de begrensde Windows-exitwait.

## Uitvoering

Definitieve opdrachten, absolute executable, cwd, tijden, exitstatus en hashes:
[validation.json](m6d-closeout-local/validation.json). Alle negen exit 0.

| Opdracht | Resultaat |
|---|---|
| `python -X utf8 -B -u tools/test_m6c.py` | 35 groepen / 31 VM; 30 Rust- en 48 Python-tests |
| `python -X utf8 -B tools/capture_m6c.py --destination docs/verification/m6d-closeout-m6c` | bron/raw/image/hostlogbinding groen |
| `python -X utf8 -B -u tools/test_m6b.py` | 18 groepen / 14 VM; 22 Rust- en 30 Python-tests |
| `python -X utf8 -B tools/capture_m6b.py --destination docs/verification/m6d-closeout-m6b` | afzonderlijk archief; crashstatus behouden |
| `python -X utf8 -B docs/verification/m6d-review/audit_m6d_native.py --self-test` | 16 synthetische hostcontroles |
| `python -X utf8 -B docs/verification/m6d-review/audit_m6d_native.py --reconstruct docs/verification/m6d-review/m6d-actual-journal-records.json` | 16 historische imagehashes uit tekst herbouwd; geen nieuwe guest |
| `python -X utf8 -B -m unittest discover -s tests -v` | latere definitieve 173 Python-tests; [receipt](m6d-closeout/build04-host-tests.json) |
| `python -X utf8 -B docs/verification/m6d-closeout/check_native_source.py` | 12 bronhashvergelijkingen; [receipt](m6d-closeout/native-source-binding.json) |
| `python -X utf8 -B tools/progress.py --check` | vaste score 14/100 |

Nieuwe archieven: [M6c/M6d](m6d-closeout-m6c/capture.json),
[M6b](m6d-closeout-m6b/capture.json). Samen 45 echte VM-starts, 20 koude
recoveries, 41 raw exits 33 en vier externe Windows-kills met raw 3765043201.
Hostfixtures, audits en reconstructies tellen niet als nieuwe VM's.

## Behouden fouten en herstel

De [eerste gate](m6d-closeout/validation.json), [failed suite](m6d-closeout/failed-suite.json)
en [runrecord](m6d-closeout/failed-normal-result.json) blijven failed.
De gast gaf raw33 en alle markers, maar de image had bij de strikte guard
werkelijk twee hardlinks. De [onmiddellijke trace](m6d-closeout/guard-trace.json)
bewaart `fsutil hardlink list`, exit 0, met een naam onder `.tmp.driveupload`.
Welk proces die link maakte is niet vastgesteld. Geen guard versoepeld.
De definitieve gates draaiden op identieke bron buiten synchronisatiemappen:
`C:/Users/simon/Documents/Codex/nieuw-os-runtime-review-20261028`.
De originele diagnostische helper staat als `diagnostic-helper.original.py`
in dit archief; voor herhaling hoort hij onder `build/diagnose_volume_guard.py`.

Bij Windows ERROR_ACCESS_DENIED wacht de runner nu daadwerkelijk maximaal de
bestaande cleanupgrens van 10 seconden op exit. Natuurlijke exit telt niet als
externe kill; nog levende processen en andere foutcodes blijven fouten.
Primaire bron: Microsoft, *TerminateProcess*, bijgewerkt 30 december 2025,
[documentatie](https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-terminateprocess),
gecontroleerd 28 oktober 2026.

De eerste brede Python-suite had 166 tests met twee errors en één failure:
synthetische fixturewrites veranderden LF in CRLF. Vier writes gebruiken nu
expliciete UTF-8-bytes. Twee tests bewaken correct CRLF-behoud én afwijzing van
gewijzigde serialbytes. De auditor heeft nu 22 tests, waaronder vier bronmutanten
met ontbrekende/te late validatie. Productieguards zijn ongewijzigd.

## Review en grenzen

Vijf eerste CodeRabbit-punten zijn hersteld. De vervolgronde vroeg volledige
hashopdrachten, aparte Rust-/Python-aantallen en controle op validatorbedrading;
de bewaarde scripts, receipts en negatieve mutanten dekken die punten.
De historische R08-review is intact. Nieuwe scope en bronhercontrole:
[FINAL-REVIEW](m6d-review/FINAL-REVIEW.md). De actuele bewijsindex bewaart
oude en nieuwe hashes traceerbaar; oude audituitvoering is niet herschreven.

De tweede AI-controle toetste alle 45 rawrecords, 395 artifacthashes,
86 raw-loghashes, 24 capturefiles en 20 cold-links. Een eigen decoder zonder
projectimports controleerde 43 imageparen: 40 geldige eindimages met 270 records
(inclusief herstelduplikaten), drie corrupte/legacy-afwijzingen en vijf
negatieve hostvectoren. Dit is geen menselijke audit of formeel bewijs.

De volledige historische Linux-rawaudit is niet opnieuw uitgevoerd. Twee
nieuwe M1-runs missen een eigen per-run-QEMU-hash; 43 M6-records hebben die wel.
Hun commandopad/profiel/ruwe logs/artifacthashes zijn gecontroleerd; preflight
vervangt de ontbrekende hashmeting niet. Geen fysieke stroomuitval, IOMMU,
SMP- of productiegeschiktheidsclaim. Herstel start nog een nieuwe VM.

Main 5a58a743 is treegelijk aan ancestor e816dfd van de ontwikkellijn.
Integratie vanaf main behoudt de gecontroleerde bronboom, uitgezonderd twee
al eerder verwijderde oude M5-logs. Geen bestaand main-bewijs vervangen.
Tijdens de uitvoering zijn #11 en #12 extern gemerged. Nieuwe main 1f24f994
is treegelijk aan de reeds gecontroleerde M6b-tip 1bc2901. De normale
main→integratie-merge behoudt deze nieuwe ouder; zijn bronboom blijft gelijk
aan closeout f3b15c6, uitgezonderd de twee bewust verwijderde oude M5-logs en
deze expliciete documentwijzigingen. Een gerichte revert van de uiteindelijke
squash is de herstelroute, zonder Git-reset.
De score blijft **14% → 14%**. Catalogusherstart K05/S08 is volgende native werk.
