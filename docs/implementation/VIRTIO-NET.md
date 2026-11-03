# FR-NDRV — native virtio-net-bouwsteen

Taakcontract, 29 oktober 2026. Eigenaar: `network_nas_ai`; integratie: hoofdagent.
Schrijfgebied: `drivers/virtio-net/**` en dit document. Geen kernel-, syscall-,
rootworkspace- of capabilitywijziging in deze commit. De hoofdagent koppelt de
driver later aan een native broker en netwerkdienst. N01 blijft open.

## Profiel en interfaces

- Eigen modern PCI-driver voor QEMU q35, vendor `1af4`, device `1041`; één CPU,
  256 MiB identiteit-gemapte RAM, MMIO supervisor/UC. De driver scant
  segment 0 via PCI configuration mechanism 1 en selecteert via
  `DriverRegistry`; de runtime is niet meer aan BDF `00:07.0` gebonden.
  Hergebruik de bestaande `virtio-core`-capabilityparser via lokale
  pathdependency.
- De huidige scanner leest maximaal 256 conventionele busnummers, 32 devices
  per bus en alleen functies 1–7 als functie 0 het multifunction-bit zet.
  Niet-aanwezige functies en niet-type-0 headers worden overgeslagen. Meer dan
  één matchende VirtIO-netfunctie wordt fail-closed geweigerd: deze kernelbroker
  heeft nog capaciteit voor één NIC. ACPI-rootbridges, andere PCI-segmenten,
  ECAM, bridge-topologie en hotplug zijn nog niet geïmplementeerd.
- Vereist `VERSION_1` en `MAC`; onderhandel optioneel `STATUS`. Geen checksum-,
  segmentatie-, mergeable-buffer-, multiqueue-, indirecte-descriptor- of
  controlqueuefeatures. MTU 1500, Ethernet maximaal 1514 bytes, geen FCS.
- Moderne virtio-net-header is **12 bytes**, ook zonder `MRG_RXBUF` bij
  `VERSION_1`. Twee split queues: RX 0, TX 1; ieder acht descriptors, aanvankelijk
  één gepubliceerd buffer per queue. Buffervergroting/meer outstanding frames
  blijft een latere prestatiestap, geen permanente productlimiet.
- `Discovery`, `NetPool`, `Device::initialize`; daarna `post_rx`, `poll_rx`,
  `send`, `poll_tx`, `link_up`, `begin_reset`, `reset_acknowledged`.
  `send` bevestigt uitsluitend queuepublicatie; `poll_tx` uitsluitend device-
  completion, nooit aflevering bij een peer. Geen ingebouwde echo of fake ACK.

## Eigendom en fouten

- Vier verschillende uitgelijnde en permanent gepinde RAM-pagina's: RX-ring,
  TX-ring, RX-bounce, TX-bounce. Per ringpagina: descriptors op 0, available
  op 256, used op 512; alle gebieden passen binnen hun eigen pagina.
- Geen apppointer, protocolparser, sleutel of routepolicy in de driver. De
  caller houdt exclusieve supervisor-I/O-eigendom; bufferinhoud wordt via
  volatile bytekopieën overgedragen, niet via gedeelde Rust-referenties naar DMA.
- Descriptor-ID, wrapping used-indexdelta, gebruikte lengte en netwerkheader
  worden vóór kopiëren gevalideerd. Geen ontvangen offload die het profiel
  niet kan verwerken; onvoldoende outputruimte geeft geen gedeeltelijke kopie.
- TX heeft een monotone deadline. Een timeout of protocolfout stopt de driver,
  vraagt reset en schakelt PCI bus mastering uit. Een TX kan dan al verstuurd
  zijn; geen automatische replay. Een lege RX-queue is normaal en geen defect:
  aanvraagtimeouts worden door de broker afgehandeld terwijl de RX-pagina gepind
  blijft. Een gepubliceerd RX-buffer kan niet zonder devicecompletion/reset
  worden teruggenomen.
- Reset geldt pas als bevestigd bij device-status nul én bus mastering uit.
  Tot die bevestiging geen hergebruik; dit profiel houdt alle DMA-pagina's
  bovendien tot reboot gepind, ook na mislukte initialisatie. Geen IOMMU- of
  bescherming tegen een kwaadwillend fysiek apparaat geclaimd.
- De PCI-configadapter bewaart het gematchte adres in de driverinstantie;
  configuratie- en busmasterwrites zijn daardoor aan dezelfde functie gebonden.
  Kernelcode blijft eigenaar van RAM-paginatoewijzing en de beperkte BAR-MMIO-
  mappings. Dit is nog geen generieke resourcebroker of userspacedriver.

## Acceptatie en bewijsgrens

Hostregister-/DMA-modellen oefenen dezelfde driverlogica: features, config-
generation, descriptorpublicatie, echte bytekopie, ringwrap, verkeerde index/ID/
lengte, offloadweigering, kleinere output, TX-deadline en reset-ACK. Label deze
tests als **model**, niet als echte NIC-uitvoering. Compileer ook werkelijk naar
`x86_64-unknown-none`; Clippy en formattering blijven verplicht.

Native vervolg: kernel koppelt gepinde pagina's en geautoriseerde processen;
QEMU met een afgebakende testpeer moet echte TX/RX bewijzen, inclusief ontbrekend
apparaat, weigering, timeout en herstel. Geen hostnetwerkstack die ontbrekende
OS-functionaliteit uitvoert. Pas daarna ARP/DNS en de overige N01/N02/N03-gates.

## Bronnen

- OASIS, *Virtual I/O Device (VIRTIO) Version 1.2*, split virtqueues en
  netwerkdevice §5.1; bestaande projectbasis `virtio-core`/blokdriver.
  https://docs.oasis-open.org/virtio/virtio/v1.2/virtio-v1.2.html
- QEMU Project, tag `v10.0.2`, `hw/net/virtio-net.c`,
  `virtio_net_set_mrg_rx_bufs`: `VERSION_1` kiest de 12-byte
  `virtio_net_hdr_mrg_rxbuf`, afzonderlijk van `MRG_RXBUF`.
  https://github.com/qemu/qemu/blob/v10.0.2/hw/net/virtio-net.c
  Primaire emulatorbron gelezen 29 oktober 2026; geen QEMU-code overgenomen.

Uitvoering, exacte bronhashes en niet-uitgevoerde native checks worden na de
bouwstap vastgelegd in `drivers/virtio-net/verification/`.

## Bouw en integratie

De nieuwe crate heet `virtio-net-driver` en hangt uitsluitend af van de bestaande
lokale `virtio-core`. De hoofdagent voegt `drivers/virtio-net` aan de bestaande
rootworkspace en lockfile toe bij integratie. Voor de geïsoleerde agentbuild is
precies die member tijdelijk alleen in de eigen worktree toegevoegd; beide
gedeelde bestanden worden vóór deze deelcommit bytegelijk hersteld. Er komt geen
nieuwe externe dependency of buildscript bij.

Na workspace-integratie zijn de gerichte controles:

```sh
cargo +1.90.0 test -p virtio-net-driver --locked --offline
cargo +1.90.0 clippy -p virtio-net-driver --all-targets --locked --offline -- -D warnings
cargo +1.90.0 build -p virtio-net-driver --lib --release --target x86_64-unknown-none --locked --offline
cargo +1.90.0 fmt -p virtio-net-driver --check
```

`post_rx` publiceert één volledig RX-buffer; `poll_rx` retourneert pas na een
gevalideerde echte used-completion een frame. Bij een te klein outputbuffer
blijft dezelfde completion beschikbaar voor een grotere buffer; er wordt nog
niets gekopieerd of opnieuw gepubliceerd. `tx_pending` en `rx_posted` tonen
alleen de lokale queuefase, geen verbindingsstatus.

Een veranderde used-index terwijl de queue idle is wordt vóór nieuwe
publicatie geweigerd: een oude/ongevraagde completion mag niet aan het volgende
verzoek worden toegeschreven. `DEVICE_NEEDS_RESET` of een andere ongeldige
device-status stopt de muterende/poll-paden; `link_up` retourneert `Io` zolang
die toestand bestaat. Een link-bit alleen betekent geen gereed apparaat.

De compacte uitkomst staat in
[`driver-acceptance.json`](../../drivers/virtio-net/verification/driver-acceptance.json).
De hosttests zijn expliciet register-/DMA-modellen; fysieke PCI, native RX/TX,
QEMU-peerconnectiviteit en rechtenafdwinging zijn voor deze slice niet getest.
