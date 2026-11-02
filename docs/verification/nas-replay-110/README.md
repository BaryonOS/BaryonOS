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
geeft eveneens `Stale`. De koude run deed 0 writes en 0 flushes. Tijdens de gate
bevat de private handoffmap (`/tmp/nos-nas-replay-*`) het ruwe tokenpaar in
`payload.bin` en `cold-input.bin`. De map en bestanden zijn privé en de map
wordt na de gate verwijderd. Gearchiveerde rapporten binden het paar met
SHA-256 in plaats van tokenbytes; `SHA256SUMS` bindt de bestanden.

Niet als native run uitgevoerd: koude run zonder of met afwijkende invoer
(alleen kernelcode `REPLAY_INPUT` en runner-hosttests). Geen claim over
netwerk/TLS, E2EE, fysieke opslag of volledige P04-acceptatie.

## Herkomstnotitie integratie 3 (31 oktober 2026)

Onafhankelijke AI-review (geen menselijke audit) heeft de fingerprint
herberekend: PR-head `fa97c692` geeft `5263a105…`; dezelfde head met
`services/nas/native-probe/CONTRACT.md` van main geeft exact `7255534f…`.
Kernel-, gast- en runnercode zijn dus precies zoals getest; alleen
CONTRACT.md is na deze run bijgewerkt. De brongebonden native proef slaagde op
`fa97c692` en opnieuw op PR #120-head `562be4d` (workflowrun
[41511329639](https://github.com/simonvanieperen/nieuw-os/actions/runs/41511329639)).
Deze runs zijn van vóór de huidige serial-archiveringsfix en bewijzen die fix
niet. De nieuwe runner controleert beide tokenhelften en archiveert alleen
gefilterde serialmarkers; exacte-head CI en een native herproef zijn nog vereist.
