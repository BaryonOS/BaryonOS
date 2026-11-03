# OpenZFS — historische uitvoering, actuele uitvoering afgeschermd

Alle oorspronkelijke commando's staan als `*.historical.txt`, bytegelijk
gebonden aan `archive-manifest.json`. Dit zijn **geen uitvoerbare scripts**;
voer ze niet via bash/Python uit. De [oude toelichting](README.md.historical.txt)
blijft historische brondata, inclusief toenmalige `/tmp`-uitzondering.

<!-- coderabbit-ai-generated:v1 -->

<!-- coderabbit-ai-generated-section:v1:56853b20abcce11034c8ccf90ebdf64ac9b7087653645a37ce1f2b76ede41ce8 -->

[COMMANDS.md](COMMANDS.md) corrigeert de verwisselde inventaris zonder
historische scriptbytes of logs te veranderen. Ook de `failed_setup_attempts`
in het historische `evidence.json` verwisselen die twee logverwijzingen;
de actuele tabel is leidend voor die koppeling. De aangepaste scriptkopieën
zijn geen goedgekeurde replayroute. Geen nieuwe no-leak-, bestandsherstel-,
native- of fysieke acceptatie.

<!-- coderabbit-ai-generated:v1 -->

<!-- coderabbit-ai-generated-section:v1:06a23f60d67d4d418570b792ff86c22fa6661e3f66e62182a75aae9477bb6018 -->

De oorspronkelijke `logs/package-metadata.txt` bewaart zes onafgesloten
`Original-Maintainer`-velden. De lokale Ubuntu `apt-cache show` en het op de
vastgelegde SHA256 gecontroleerde `zfsutils-linux`-pakket van dezelfde versie
bevatten ook die onvolledige waarde. De originele snapshot is daarom niet
stilzwijgend aangepast. Deze contactvelden zijn niet volledig uit dit bewijs
te verifiëren.

<!-- /coderabbit-ai-generated:v1 -->

<!-- /coderabbit-ai-generated:v1 -->

De [actuele route](../COMMANDS.md) kan uitsluitend zes exact gepinde
offlinepakketten controleren en veilig uitpakken. Een extra `.deb`, verkeerde
hash/identiteit, link, gedeelde invoermap of bestaande werkruimte faalt.
Er wordt niets gedownload, geïnstalleerd of automatisch uitgevoerd.

**OpenZFS-uitvoering blijft geblokkeerd.** Upstream OpenZFS `zfs-2.4.1`,
`cmd/ztest.c:8717–8723`, `setup_data_fd()`, gebruikt hardcoded
`/tmp/ztest.data.XXXXXX` met `mkstemp`, ongeacht TMPDIR.
[Primaire bron](https://github.com/openzfs/zfs/blob/zfs-2.4.1/cmd/ztest.c#L8717-L8723),
gericht gecontroleerd op 2 november 2026. Zonder een afzonderlijk
geaccepteerde isolatieroute is een uitsluitend project-lokale create/crashreplay
dus niet geïmplementeerd. De runner weigert `--backend openzfs --execute`
vóór packages, werkruimte of foreign processen worden aangeraakt.

De toekomstige kopie-/zdb-/packagingbouwstenen zijn alleen met synthetische
hostfixtures getest: kopiehashes moeten overeenkomen, `zdb` krijgt de
gecontroleerde kopiemap en poolnaam `nieuwos_n3`, niet de historische GUID.
Packaging maakt zelf `evidence/images` aan. Dit is geen volledige
OpenZFS-replay of nieuwe filesystemuitkomst.
