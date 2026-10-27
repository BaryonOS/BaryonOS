# Volledige M6a-poort op Linux

Eén nieuwe complete uitvoering op basis `10b32e31671827f2bd63444113b981ff15bfbb73`:
54/54 controlegroepen, 49 afzonderlijke echte VM-invocaties, zes dienstcrashes,
zes externe QEMU-kills en 17 afzonderlijke koude recoveries. Gate en capture
beide exit 0. De extra eerste smoke is niet bij de gate opgeteld. Dit resultaat
sluit de vroegere onderbreking na 43/54 groepen op de ongewijzigde baseline af.

`summary.json` en de twee commandrapporten beschrijven de werkelijke uitvoering.
`guest-runs.json` bewaart ruwe uitvoer, exitstatus, commands, artifacthashes en
schijforacles. `suite.json` bindt de gehele volgorde aan één bronfingerprint.
Alle 15 bestanden in `capture-generated-files.json` zijn vóór kopiëren opnieuw
op SHA256 gecontroleerd. Historische Windowsbewijzen blijven ongewijzigd in
`../m6a/` staan. De Linux-toolchain staat in `../native-linux-toolchain/`.

Deze gate geldt voor M6a op de genoemde basis. De M6b-feature en M1–M5-regressie
op nieuwe bron hebben aparte rapportmappen. Geen optelling van oude deelruns,
geen fysieke stroomuitval- of vijandige-DMA-claim.
