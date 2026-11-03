OpenZFS N3 — hostproef afgerond; native backend en bestandsherstel NIET GEACCEPTEERD

Datum: 2 november 2026. Uitvoering: WSL Ubuntu 26.04.1, amd64. Dit dossier bevat echte uitvoering van bestaande OpenZFS 2.4.1-1ubuntu5.1-userspacebinaries. Het bevat geen eigen filesystemdriver, kernelport of bewijs van uitvoering in de nieuw-os-kernel.

Resultaat

| Proef | Werkelijk resultaat | Betekenis |
|---|---|---|
| ztest-aanmaak | exit 0; 2 complete passes; 0 kills; 42,94 s; max RSS 243.240 KiB | Echte mirrorpool op regular files gemaakt. |
| Verse zdb import/check | exit 0; 1.141 blockpointers; alle data- en metadatachecksums gecontroleerd; expliciet no leaks | Pool/objectconsistentie in een nieuw hostproces aangetoond. |
| Tweede baselinecheck zonder -Y | exit 0; dezelfde 1.141 blockpointers; no leaks | De positieve baselinecontrole hangt niet af van all-reconstruction. |
| ztest -E | exit 3; child SIGABRT; no pools found | Hergebruik via deze harness faalt. Bronpad voor active-check gebruikt /dev/zfs; er is geen module geïnstalleerd of vervangende shim gemaakt. |
| Crashrun | ztest exit 0; werkelijk 3 SIGKILL; 0 complete passes; 34,53 s; max RSS 135.328 KiB | Processcrashes uitgevoerd; geen fysieke stroomuitval. |
| Verse zdb na crashes | exit 0; 519 blockpointers; checksumfouten 0, maar gemeld allocatieverschil/leaked 9.216 bytes | Niet geaccepteerd onder de strenge no-leak oracle. Oorzaak en betekenis blijven onopgelost. |
| Raw DMU-backupstream | exit 134; assertion; uitvoer 0 bytes | Geen bruikbare export; receive/restore niet aangetoond. |
| Verbose DMU-objectdump | exit 134; ARC-assertion | Geen payloadextractie aangetoond. |

De twee asserts en het allocatieverschil zijn bevindingen van deze precieze hostproef. Zij rechtvaardigen geen algemene claim dat productie-OpenZFS of zijn kernelimplementatie defect is. Een exitcode 0 geldt niet als voldoende herstelbewijs wanneer de inhoudelijke controle niet aan de gekozen oracle voldoet.

Bewijsgrens

ztest produceert DMU-objectsets van type OTHER. De ZPL-root van deze baseline is leeg: nul objecten. Er is dus geen normale bestandsboom met namen, mappen en zelf aangeleverde payload waarvan hashes na export zijn vergeleken. Poolimport, checksums en objectconsistentie zijn apart beoordeeld van gewone bestandsrecovery. Automatische/handover-sync, encryptiesleutelverlies, volle schijf, miljoenen bestanden, SSD-slijtage, een tweede fysiek apparaat en persistente self-healing zijn niet getest.

De gemeten ztest_fault_inject-teller is nul in beide runs. Er is geen externe blokcorruptie geïnjecteerd en geen succesvolle mirrorreparatie geclaimd. De crashrun gebruikt SIGKILL en verifieert geen device write cache, flush-ACK-contract of hardware-powerloss.

De eerste verse zdb-opdracht bevatte -Y: alle reconstructiecombinaties voor indirect split blocks. Dit is geen rewind. Er was een externe timeout. Een aanvullende baselinecheck zonder -Y slaagde eveneens. De postcrashcontrole zonder -Y meldde hetzelfde allocatieverschil.

Engine en grenzen

Zes officiële Ubuntu-pakketten zijn met apt download opgehaald en met dpkg-deb -x lokaal uitgepakt. Alle zes SHA256-hashes zijn vergeleken met de vastgelegde lokale apt-metadata. De .deb-bestanden, volledige licentievermelding, afhankelijkheden, bronverwijzingen en executed-binary-hashes zijn bewaard. De bronbestanden op upstreamtag zfs-2.4.1 verklaren het gedrag, maar zijn niet identiek verklaard aan alle Ubuntu-patches.

Er is geen apt install, modprobe, mount, loopdevice, fysieke schijfhandeling of repositorywijziging uitgevoerd. Schrijfacties bleven in de twee geautoriseerde proefmappen. De vooraf geautoriseerde uitzondering is ztests hardcoded /tmp/ztest.data.XXXXXX: een mkstemp-bestand dat direct wordt geünlinkt. De engine is niet gepatcht.

Lange opdrachten hadden timeouts van 30–120 seconden met 5 seconden killgrace, core dumps uit en een virtuele-adresruimtelimiet van 8 GiB per proces. Er was geen gezamenlijke harde RSS-cap. MALLOC_ARENA_MAX=2 en ZFS_STACK_SIZE=131072 beperkten allocator-/threadreserveringen. Het testworkload gebruikte twee threads; de library heeft aanvullende threads. De genoemde RSS-/looptijden zijn metingen van deze kleine hostproef, geen sizingadvies of prestatieranglijst.

Eerdere mislukte aanroepen zijn behouden: onjuiste -C-syntax uit de helpinterpretatie, niet ondersteunde dedup-classoptie, niet beschikbare ARC-tunables en een pthread_create-fout bij een 4 GiB-adresruimtelimiet. De succesvolle opdracht gebruikt -C special=off en 8 GiB; er is niets aan de engine veranderd.

Native toepasbaarheid

De uitgevoerde binaries gebruiken de Linux ELF-interpreter, glibc, threads en andere hostbibliotheken; zie ldd-logs. Bovendien raakt ztest -E in deze versie een /dev/zfs-active-checkpad. Dit is bewijs dat deze uitgepakte hostbinaries geen rechtstreeks bruikbare nieuw-os-backend vormen. Er is geen native build of bronport uitgevoerd; de haalbaarheid van een aangepaste port is hiermee niet beslist. De afzonderlijke native-agent gebruikt een baselineleaf alleen voor foreign-formatgedrag, niet als volledige native bestandssysteemtest.

Artefacten

- evidence.json: machineleesbare gates, werkelijke exitcodes, tijd/RSS, topologie, datasetmanifest, bron- en pakketprovenance.
- artifact-sha256.json en SHA256SUMS.txt: inhoudshashes van alle dossierbestanden behalve de manifests zelf.
- COMMANDS.md en genummerde scripts: exacte uitvoerbare proefopdrachten; logs/: onbewerkte stdout/stderr, exitcodes en time-uitvoer.
- images/nieuwos_n3.0a en images/nieuwos_n3.1a: onveranderde quiescent baselinekopieën, elk 268.435.456 bytes; pool GUID 9169139500157372876, één mirror, ashift 9, labelversie 5000, beste uberblock txg 526.
- images/crash-pool.tar.gz: bewaarde postcrashpool voor latere analyse; geen herstelclaim.
- packages/: gedownloade .deb-bestanden; sources/: geraadpleegde upstreambronkopieën; licenses/: distributielicenties; exports/ds0-raw.zfs: de mislukte lege export.

Een afzonderlijk mirrorlid kan in principe alle mirrordata dragen; zelfstandige degraded import is hier niet getest. Herhaal scripts alleen als wegwerpproef: de create-scripts maken hun genoemde testpool opnieuw aan. Er hoeft geen hostconfiguratie te worden teruggedraaid. Herstel naar de onaangeroerde proefbaseline kan door de bewaarde twee imagekopieën naar een nieuwe proefmap te kopiëren; daarmee is nog geen gebruikerbestand hersteld.

Officiële documentatie: https://openzfs.github.io/openzfs-docs/man/v2.4/1/ztest.1.html en https://openzfs.github.io/openzfs-docs/man/v2.4/8/zdb.8.html. Exacte bron-URLs en hashes staan in evidence.json.