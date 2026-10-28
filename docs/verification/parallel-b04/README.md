# B04 — gepubliceerde WRITE bij M6b-timeout

## Resultaat en afbakening

De onafhankelijke M6b-schijforacle accepteert voor `write-timeout`,
`reset-timeout` en `write-poll-limit` precies twee mogelijke uitkomsten:

- de volledige oorspronkelijke normale fixture, met sequence 0;
- uitsluitend het eerste canonieke PUT-record: sequence 1, request-id 10,
  object 0, revision 1, met de 128 verwachte payloadbytes.

Beide uitkomsten vereisen de oorspronkelijke normale fixture als `before`.
De bestaande controles op alle bytes buiten het journal, aaneengesloten
records, eigenaar, revisies, header/padding, lengte en FNV blijven gelden.
FNV controleert beschadiging; het is geen authenticatie. Een tweede record,
een andere mutation of een gastclaim van zichtbare state/ACK wordt afgewezen.
De bestaande FLUSH-timeouts blijven precies één record vereisen.

Dit is een reparatie van een hostcontrole. Kernel, driver, service, capability-
en schijfformaat zijn niet gewijzigd. Deze bundel bevat geen nieuwe native
of fysieke hardwaretest. De roadmapscore verandert niet door deze reparatie.

## Vastgestelde bronsemantiek

Onderzochte basis: `41bac3c173d464a286017d31558335d70899c434`.
De individuele SHA-256's staan in `green.json`.

- `drivers/virtio-blk/src/lib.rs`, `publish_prepared`: de driver schrijft
  `avail.idx` voordat hij de optionele notificatie verstuurt. Zijn contract
  vermeldt expliciet dat een polling device de publicatie al kan zien.
- `kernel/src/m2/m3/m6b.rs`, `submit`: de policyticket krijgt eerst de status
  Published; daarna wordt de eerste WRITE gepubliceerd met onderdrukte
  notificatie voor de drie genoemde cases. Er is geen terugname van de
  descriptorpublicatie.
- `userspace/m6b_probe.rs`, `uncertain_client`: alleen domein 0 begint met
  request 10, object 0 en de eerste PUT. De peer wacht tot de echte Unknown-
  reply is waargenomen. `value(0, 1)` is exact
  `(0x31 + 7 + index * 3) & 255` voor index 0 tot en met 127.
- `services/objects/src/user_service.rs`: alleen voltooide WRITE én FLUSH
  leiden tot `current.commit`; een fout sluit de service voor verdere geldige
  bewerkingen met `Status::Unknown`. Toegangsvalidatie blijft vóór die guard.
- `tools/run_m6b.py`: de runner bewaart het werkelijke before-image en weigert
  herhaling van een mutatorstage op een reeds gebruikte volume-identiteit.
- `tools/m6c_disk.py`: de bestaande M6c-oracle houdt voor zijn eigen
  gepubliceerde rename-WRITE al rekening met de oude of nieuwe toegestane
  prefix. De M6b-correctie volgt zijn eigen hierboven vastgestelde scenario.

Bronnen zijn de gelijknamige repositorybestanden op deze basiscommit,
Nieuw OS-project, gelezen op 2026-10-28:
[bronboom](https://github.com/simonvanieperen/nieuw-os/tree/41bac3c173d464a286017d31558335d70899c434).
De review van CodeRabbit in PR #14, thread `PRRT_kwDOVZC91s6xLzVA`, was
aanleiding voor deze controle; de code bepaalt de geaccepteerde uitkomsten.

## Uitgevoerde controles

Reproduceer vanuit de repository:

```sh
python3 -B -m unittest discover -s tests -p test_m6b_disk.py -v
git diff --check -- tools/m6b_disk.py tests/test_m6b_disk.py
```

`red.json` en `red-old-oracle.txt` binden dezelfde nieuwe tests aan de oude
oracle. De suite voerde 16 testmethoden uit en faalde op precies de drie
toegestane sequence-1-uitkomsten. `green.json` en `green-oracle.txt` binden
dezelfde testbron aan de gerepareerde oracle: 16 testmethoden geslaagd.

De nieuwe grensgevallen testen beide toegestane schijven voor alle drie
WRITE-timeouts en behouden de strengere FLUSH-uitkomst. Negatieve gevallen
dekken een ander eigen object, een vreemd domein, verkeerde revision of
request-id, DELETE, verkeerde of korte waarde, replay, opnieuw berekende
checksums bij foutieve padding, beschadigde checksum en veranderde canaries
vóór of na het journal. Ook een niet-verse before-image, een corrupt fixture-
type en volledige of gedeeltelijke live-state/ACK-claims worden geweigerd.
De bestaande test die iedere recordbyte afzonderlijk beschadigt blijft groen.

De testvolumes worden uitsluitend in afzonderlijke tijdelijke directories
gemaakt en door hun eigen fixturemanifest bewaakt. Er zijn geen echte disks
of gedeelde buildimages gebruikt. De geïntegreerde native M6b-gate wordt
door de hoofdagent op de samengevoegde bron uitgevoerd; dit hostbewijs
vervangt die gate niet.
