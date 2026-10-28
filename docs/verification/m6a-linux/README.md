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

## Correctie van de bewijsverwijzing

[`suite.json`](suite.json) en [`capture-generated-files.json`](capture-generated-files.json)
blijven de bytegelijke oorspronkelijke capture. De afgeleide leesindex
[`suite.corrected.json`](suite.corrected.json) corrigeert uitsluitend de verwijzing
voor `m6-user-artifact-tests` (`M6A-USER-ELF`) van het lege stdout-bestand naar het
al gearchiveerde [stderr-bestand](m6a-user-artifact-tests.stderr.txt).
Dat log bevat de twaalf testresultaten; het bijbehorende
[commandrapport](m6a-user-artifact-tests.json) registreert exit 0.

De herkomstmetadata van de leesindex vermeldt de oorspronkelijke suite, haar
SHA256, de archiefbasis `3f308b07e0bab445d1ea926561eab8d8e01c86a4` en de
uitvoeringsbasis `10b32e31671827f2bd63444113b981ff15bfbb73`. Uitslagen, tijden en
bronfingerprint zijn overgenomen uit de oorspronkelijke uitvoering. De SHA256
van de oorspronkelijke suite en het stderr-bestand zijn voor deze correctie
opnieuw vergeleken met het bestaande capturemanifest; beide komen overeen.

Deze correctie wijzigt geen bestand uit de oorspronkelijke set van vijftien
capturebestanden; de bestaande capturehashes blijven daardoor geldig. De
afgeleide leesindex is geen oorspronkelijk capturebestand en wordt niet aan
het historische manifest toegevoegd. Er zijn geen nieuwe hosttests of
VM-runs uitgevoerd en er wordt geen nieuw bewijs voor de huidige kernel geclaimd.
