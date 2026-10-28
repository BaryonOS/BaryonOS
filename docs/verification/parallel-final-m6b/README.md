# B05 — volledige M6b-integratiegate

**Geslaagd: 32 groepen en 28 afzonderlijke echte QEMU-invocaties.** Zowel
`tools/test_m6b.py` als de ongewijzigde `tools/capture_m6b.py` eindigde met
exit 0. De uitvoering liep op 28 oktober 2026 van 15:36:30 tot 15:40:18 UTC.
De gate duurde 227,54 seconden; de onafhankelijke capture 0,52 seconden.

## Bron en omgeving

- Uitvoeringscommit: `3e41bbf93822f7044c6020259f4a752925e1966d`.
- Bronfingerprint v3:
  `307c828e9ce7bf49a4e809859ca77ecd332015572dd37e96fb580de98951aa60`.
- De commit en fingerprint zijn vóór start gecontroleerd. De bronfingerprint
  bleef gelijk gedurende suite en capture; hosttests hebben daarnaast hun
  eigen vaste SHA-256-map in `suite.json`.
- Officiële standalone Rust 1.90.0, met werkelijke `none`- en UEFI-targetlibs.
  De strikte inventaris bleef eerlijk `rustup.available=false` rapporteren.
- QEMU 7.2.22, expliciet `pc-q35-7.2`, TCG met één CPU en 256 MiB;
  gepinde OVMF CODE/VARS en `-nic none`. Dit is het vastgelegde ontwikkelprofiel,
  geen uitvoering op fysieke hardware of op het 10.0-referentieprofiel.
- Eigen buildmap `build/parallel-final-m6b`, eigen targetmap
  `/tmp/nieuw-os-par-final-m6b-target`, twee Cargo-buildjobs, offline cache.

De ongewijzigde repo-implementatie is de primaire bron voor de betekenis van
de proeven: Nieuw OS-project, bovenstaande commit, gelezen/uitgevoerd op
2026-10-28, [bronboom](https://github.com/simonvanieperen/nieuw-os/tree/3e41bbf93822f7044c6020259f4a752925e1966d).

## Gecontroleerde mogelijkheden en grenzen

De echte objectdienst schrijft, leest, wijzigt en verwijdert kleine objecten
in twee gescheiden domeinen. Revisies en tombstones worden na een nieuwe
VM-start uit dezelfde schijfbytes hersteld. Twee normale runs gebruiken
dezelfde kernel-/loaderbinaries en verschillende willekeurige canaries;
documentwaarden zijn bewust deterministisch. De foutaanvraagproef wijst
34 aanvragen zonder I/O af en voltooit acht geldige mutaties. Het volle
journal bevat exact 32 records en overschrijft geen bestaand record.

De suite omvat 13 cold-recoveryruns, twee echte externe afbrekingen van een
nog levende QEMU-child en een laatste gewone M1-bootregressie. Recovery
gebruikt telkens precies het vorige after-image, dezelfde volume-identiteit,
een afzonderlijke VM en geen nieuwe writes. De twee hostkills hebben echte
raw-exitstatus `-9`; alle reguliere VM's eindigen met debug-exitstatus 33.

| Foutcase | Werkelijk gevonden journal-sequence | Native uitkomst |
|---|---:|---|
| `write-error` | 1 | Unknown |
| `flush-error` | 1 | Unknown |
| `write-timeout` | 0 | Unknown |
| `flush-timeout` | 1 | Unknown |
| `reset-timeout` | 0 | Unknown |
| `write-poll-limit` | 0 | Unknown |
| `flush-poll-limit` | 1 | Unknown |

Iedere foutcase bevat precies één `M6B:OUTCOME:UNKNOWN` en geen zichtbare
state-/succes-ACK-claim. De eerdere B04-reparatie staat bij de drie gepubliceerde
WRITE-timeouts ook uitsluitend het exacte eerste canonieke record toe.
Deze native uitvoering observeerde daar telkens sequence 0; de toegestane
sequence-1-variant is met de afzonderlijke B04-hostregressies gecontroleerd.
Er is geen andere schijfuitkomst of automatische replay toegestaan.

De gate voerde daarnaast 22 object-core/wire-hosttests en 38 Python-oracle-,
capture- en harnastests uit. De hosttests zijn afzonderlijk gelabeld en worden
niet als extra native runs geteld.

## Bewijs en eigen reproductie

`capture.json` bindt de volledige gevalideerde bundel. `suite.json` bevat de
32 resultaten; `guest-runs.json` bevat de ruwe stdout/stderr, exitstatussen,
exacte commandoregels, artifacts, schijforacle en recoveryrelaties van alle
28 VM's. De capture heeft de bewaarde echte image-/ELF-/firmwarebytes, ruwe
logs, vaste dekking en bronbinding opnieuw gecontroleerd. `execution.json`
bindt start/stop, commando's, omgeving en ruwe gate-/captureloghashes.

De binaire artifacts en images blijven in de geïsoleerde buildmap; het
Git-archief is uitsluitend tekst. `gate.stdout.txt`/`gate.stderr.txt` en
`capture.stdout.txt`/`capture.stderr.txt` zijn onbewerkte uitvoer. De extra
startwrapper controleert alleen de vooraf bevestigde freeze en roept de
bestaande suite en capture aan; hij genereert geen gastresultaten.

Gebruik dezelfde uitvoeringsbron en een nieuwe, nog ongebruikte build- en
archiefnaam. In deze ontwikkelomgeving:

```sh
source /tmp/nieuw-os-par-tools/env.sh
export CARGO_HOME=/tmp/nieuw-os-par-tools/cargo-home
export NOS_BUILD_DIR="$PWD/build/m6b-reproduction"
export NOS_TARGET_DIR=/tmp/nieuw-os-m6b-reproduction-target
export CARGO_TARGET_DIR="$NOS_TARGET_DIR"
export CARGO_BUILD_JOBS=2
export NOS_SDK_RUNTIME=0
python3 -B tools/test_m6b.py
python3 -B tools/capture_m6b.py --destination docs/verification/m6b-reproduction
```

Op een andere machine zijn Rust 1.90.0 met beide echte targetbibliotheken,
QEMU met het expliciet gekozen machineprofiel en een passende OVMF-paar
nodig; de opgeslagen inventaris en VM-commandoregels leggen dit vast.
De suite weigert bronwijzigingen tijdens de uitvoering.

## Resterende beperkingen en score

Dit is regressie- en integratiebewijs voor de bestaande M6b-grenzen: vier
objecten, maximaal 128 inhoudsbytes en een journal met 32 records. Het bewijst
geen grote documenten, ruimtehergebruik, native Materiaal-bewerking, encryptie,
algemeen herstelbeleid of fysieke-hardwareduurzaamheid. M6c/SDK, G0 en de
quantumport hebben hun eigen gates en bewijsbundels.

Deze gate voegt **0 nieuwe roadmap-punten** toe. Zij bevestigt de bestaande
opslagfunctionaliteit en de gerepareerde controle op de samengevoegde bron;
criteria en gewichten zijn niet gewijzigd. Publicatie en een eventuele
bewijscommit worden centraal door de hoofdagent afgehandeld. Deze agent
heeft de bevroren HEAD niet gewijzigd.
