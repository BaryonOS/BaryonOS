# #110 — exacte boot-1 NAS-tokenreplay native (normaal + koude herstart)

30 oktober 2026, branch `claude/issues-20261030` op basis `main` `e08fb0e`.
De werkboom bevatte de #110-wijzigingen nog ongecommit; de runrapporten noemen
daarom `e08fb0e` als `source_commit`. De daadwerkelijk geteste bron is
gebonden via bronfingerprint v3
`7255534f462eceba…` (volledige waarde in `nas-system-gate.json`).

Commando (lokale `.tools`-omgeving, QEMU 10.2.1, `pc-q35-8.2`, TCG, 1 CPU,
256 MiB, wegwerpimage):

```sh
python3 -B -u tools/test_nas.py
```

| Stage | Raw exit | Duur | Rollen (ok, fouten) beheerder/NAS/gast | Replay |
|---|---|---|---|---|
| normaal | 33 | 5,2 s | (6,1) (12,6) (6,5) | `NAS:REPLAY_EXPORT:PASS`, paar-SHA-256 `0fd5eb0b…` |
| koude herstart, read-only disk | 33 | 3,6 s | (4,0) (4,5) (0,5) | `NAS:REPLAY_INPUT:PASS`, zelfde SHA-256 en origin-run |

De extra foutreply bij NAS en gast in de koude run is de `COMMIT` met de
exacte oude sessie en upload, die `Stale` geeft; `READ` met de oude sessie
geeft eveneens `Stale`. De koude run deed 0 writes en 0 flushes. De private
handoffmap (`/tmp/nos-nas-replay-*`) is na de gate verwijderd. Deze map bevat
geen tokenbytes: alleen de SHA-256 van het paar, de commando's met het
private pad, de COM1-serial en de rapporten; `SHA256SUMS` bindt de bestanden.

Niet als native run uitgevoerd: koude run zonder of met afwijkende invoer
(alleen kernelcode `REPLAY_INPUT` en runner-hosttests). Geen claim over
netwerk/TLS, E2EE, fysieke opslag of volledige P04-acceptatie.

## Herkomstnotitie integratie 3 (31 oktober 2026)

Onafhankelijke AI-review (geen menselijke audit) heeft de fingerprint
herberekend: PR-head `fa97c692` geeft `5263a105…`; dezelfde head met
`services/nas/native-probe/CONTRACT.md` van main geeft exact `7255534f…`.
Kernel-, gast- en runnercode zijn dus precies zoals getest; alleen
CONTRACT.md is na deze run bijgewerkt. De brongebonden native proef is de
groene CI-gate `nas-native-normal-cold` op `fa97c692` en daarna op de
integratiehead. De run hierboven en die CI-gates gebruikten nog de eerste
lekcontrole: die zocht alleen het volledige 80-bytepaar, als hex en via
latin-1 in serialtekst die al lossy als UTF-8 was gedecodeerd, en schreef
`serial.txt` vóór de controle. INT3-FU-119 (hieronder) vervangt die controle.

## Lekcontrole sinds INT3-FU-119 (1 november 2026)

**Geïmplementeerd** in `tools/test_nas.py` (branch `int3-fu-119`, commit
`bdcf461`, basis `562be4d`):

- `execute_raw` bewaart QEMU-stdout (COM1-serial) en -stderr als exacte
  bytes. `run_volume.execute` decodeert met vervanging, waardoor ruwe
  tokenbytes verdwijnen; dat bestand is niet gewijzigd.
- `token_leaks` zoekt de sessie (`paar[:40]`) en de pending upload
  (`paar[40:]`) apart, als elk aaneengesloten 16-bytevenster (25 per token),
  zowel ruw als hex ongeacht hoofdletters, in stdout én stderr. Een volledig
  token bevat altijd zijn vensters.
- Waarom 16 bytes: echte tokens zijn `boot(32 random bytes, gedeeld door
  sessie en upload) || le64(teller)` (`encode_token` in
  `services/nas/native-core`). Elk venster bevat dus minstens 64 random bits.
  De kans op een toevallige treffer in n uitvoerbytes is daarom van orde
  n·100/2^64. Door de gedeelde prefix kan een lek van alleen de sessie ook een
  upload-label opleveren, en omgekeerd. Een schone run
  faalt dus niet bij toeval, terwijl elke kopie van 16 of meer aaneengesloten
  tokenbytes (een half token, een afgekapte print) wel gevonden wordt.
  **Niet herkend:** kortere fragmenten, meerdere losse korte stukken en
  andere coderingen (base64, decimaal, hex met scheidingstekens, UTF-16).
- `archive_output` draait vóórdat iets uit de QEMU-uitvoer in de runmap komt.
  De referentie is in de normale run de private debugcon-capture (zonder
  `NOSRPL01`) en in de koude run het paar dat vóór de start is gebonden.
  Bij een treffer, of als de capture onleesbaar is (fail closed), faalt de
  run. `serial.txt` en `stderr.txt` krijgen dan alleen een placeholder:
  `REDACTED`, labels zoals `session:serial:hex` (geen bytes), en de lengte en
  SHA-256 van de originele bytes. Ook `result.json` bevat alleen die labels.
  Een capture die niet exact één `NOSRPL01`-frame van 80 bytes is (dubbele
  export, junkprefix, verkeerde magic, te kort), wordt in zijn geheel als
  geheim behandeld: de runner doorzoekt elk 16-bytevenster van alle
  gecapturede bytes (label `capture:…`). Een lege capture heeft niets
  geëxporteerd en bewaart de serial, zodat vroege gastfouten te debuggen
  blijven; de run faalt dan op de framing. (Integratie 3, na review: eerder
  werd bij een misvormde capture alleen `capture[8:88]` doorzocht.)
- De private handoffmap: `main()` verwijdert die in `finally` bij elke
  normale of foutuitgang, ook bij onverwachte excepties. Bij SIGKILL of
  SIGTERM van de runner draait `finally` niet; de 0700-map in `/tmp` blijft
  dan achter. `private_handoff_dir()` verwijdert
  nu ook zelf een map die het weigert (binnen de checkout); voorheen bleef
  die lege map achter.

**Hosttests** (`tests/test_nas_runner.py`): een nep-QEMU op
`subprocess.Popen`-niveau levert exacte stdout-/stderr-bytes. Dat bewijst
alleen de runnerlaag, niet dat een gast draaide. Nieuw: lek van alleen de
sessie (hex), alleen de upload, hex in hoofdletters, ruwe bytes (ook niet-UTF-8),
16-bytevensters op verschillende posities (begin, midden, eind), lek in stderr, lek
in de koude run, en geen originele uitvoer of tokenvenster in enig bestand van
de testmap of handoffmap na een lekkende run. Daarnaast het opruimen van de
handoffmap via `main()` en bij weigering. Een schone run bewaart de echte
serial; een fragment van 15 bytes wordt bewust niet herkend (drempel vastgelegd).

| Controle | Commando | Resultaat |
|---|---|---|
| Rode basislijn: nieuwe tests tegen de runner van `562be4d` | `python3 -B -m unittest discover -s tests -p "test_nas_runner.py" -v` | exit 1: 28 tests, 15 failures (alle 10 nieuwe tests, subtests apart geteld), 0 errors; de 18 bestaande tests groen |
| Na de fix | zelfde commando | exit 0, 28 tests OK |
| NAS-hosttests | `python3 -B -m unittest discover -s tests -p "test_nas_*.py"` | exit 0, 32 tests |
| Native-probe-hosttests | `python3 -B -m unittest discover -s services/nas/native-probe/tests -p "test_*.py"` | exit 0, 17 tests |
| Integratie 3: misvormde-capturetests tegen de runner van `int3-fu-119` | `python3 -B -m unittest tests.test_nas_runner` | exit 1: 3 failures (dubbel frame, junkprefix, verkeerde magic) en 2 errors (nieuwe `pair_references` ontbreekt) |
| Integratie 3: na de fix | `python3 -B -m unittest discover -s tests -p "test_nas_*.py"` | exit 0, 37 tests |
| Integratie 3: native, buiten de checkout | `python3 -B -u tools/test_nas.py` (QEMU 8.2.2, `pc-q35-8.2`) | exit 0; normaal en koud `passed=True raw=33`, geen redactie |

**Native run, update integratie 3 (2 november 2026): groen.** De mislukking
hieronder kwam door de omgeving, niet door QEMU of firmware. De builder draaide
in een geneste worktree onder de checkout; Cargo voegde dan twee
`.cargo/config.toml`-bestanden samen en gaf `-Tkernel/linker.ld` twee keer
mee, wat een kapotte kernel opleverde. Dezelfde gate in een worktree buiten
de checkout, met de lekcontrole inclusief de misvormde-capturefix: normaal en
koude herstart beide `passed=True raw=33`, geen redactie (geen vals-positief
op echte serial), op QEMU 8.2.2, `pc-q35-8.2`.

**Oorspronkelijke native run (historisch, omgevingsfout):** Lokaal, QEMU 8.2.2
(`Debian 1:8.2.2+ds-0ubuntu1.18`), `pc-q35-8.2`,
`/usr/share/OVMF/OVMF_CODE_4M.fd` en `OVMF_VARS_4M.fd`, op schone commit
`bdcf461`: `python3 -B -u tools/test_nas.py` gaf exit 1. De normale stage gaf
`passed=False raw=0` na 4,1 s; de gast stopte na `KS-02:PASS` (vóór
`KS-03:PASS`, tijdens het opzetten van paging) en de koude stage is niet
gestart. Als diagnostische controle (buiten de ene gevraagde run) draaide
daarna de ongewijzigde runner van `562be4d` op dezelfde boom en omgeving. Die
faalde op precies dezelfde manier, met byte-identieke `serial.txt` (SHA-256
`7573e4c4186f61fb…`). De oorzaak ligt dus niet in deze runnerwijziging. Een
reset of triple fault door afwijkende firmware of QEMU-versie is een
**onbewezen** vermoeden. De nieuwe lekcontrole is daarom nog niet native op
een geslaagde normaal-plus-koud-run aangetoond; ook CI kon niet draaien
(Actions `startup_failure`).

**Niet gescand:** `volume.img` en `OVMF_VARS.fd` zijn door de gast
beschrijfbaar en vallen ook onder het CI-artifact `run/nas-*/`; deze
controle doorzoekt alleen serial en stderr.
