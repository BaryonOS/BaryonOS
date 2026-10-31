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
integratiehead. Bekende beperking van de lekcontrole in `tools/test_nas.py`:
die zoekt alleen het volledige 80-bytepaar als hex en schrijft `serial.txt`
vóór de controle; losse 40-bytetokens worden niet apart gezocht. Dat staat
als vervolgpunt in `docs/PROJECT-STATE.md`.
