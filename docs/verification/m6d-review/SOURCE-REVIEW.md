# R08 — onafhankelijke bronreview van native M6c-onzekerheidsafhandeling

**Conclusie:** in de beoordeelde bron is geen open blokkerende bevinding gevonden; twee concrete harnasbevindingen zijn hersteld en de twee gerichte hercontroles slagen.

Datum: 28 oktober 2026. Reviewer: afzonderlijke AI-agent `native_review`.
Dit is een AI-review, geen menselijke audit of formeel veiligheidsbewijs.
Basiscommit: `b58ae2a56be6978eab2b5ac4bb5d12e3d615865f`.
Beoordeeld: de lokale wijzigingen voor de M6d-bouwstap op de bestaande M6c-route.
De exacte SHA-256-hashes van de beoordeelde bestanden en de uitgevoerde controles
staan in `r08_review_checks.json`. De bestanden bleven tijdens die controles
ongewijzigd. De definitieve native gate is in deze notitie nog niet geaudit.

## Scope

- `kernel/src/m2/m3/m6c.rs` en `userspace/m6c_probe.rs`.
- `kernel/Cargo.toml`, `tools/build.py`, `tools/run_m6c.py`,
  `tools/m6c_disk.py`, `tools/m6c_faults.py`, `tools/capture_m6c.py`,
  `tools/test_m6c.py` en `tests/test_m6c_uncertainty.py`.
- Relevante ongewijzigde interfaces: de native catalogusadapter, wire/journal,
  de virtio-policy en de bestaande virtio-driverinjecties.

Root-, kernel-, services- en testinstructies en de relevante opslag-/beveiligings-
en testcontracten zijn geraadpleegd. De reviewer wijzigde geen repositorybron.

## Gecontroleerde invarianten

De eerste `Unknown` is aan een daadwerkelijk door de broker teruggegeven
mislukte muterende syscall gebonden. Een normale pending-poll telt niet als
terminale fout. Deze observatie geldt ook zonder ingeschakelde foutfeature.
Een onzekere mutatie wordt niet in de levende toestand gecommit en niet als
succesvolle ACK geteld. De verwachte foutresponse is canoniek leeg, inclusief
nulrevisie en afwezigheid van naam, inhoud en catalogusentries.

Na die eerste fout wordt een exacte snapshot van dienstcalls, publicaties en
succesvolle completions vastgehouden. Ook een geslaagde device-reset staat geen
herhaling, nieuw blockverzoek of antwoord uit de oude levende toestand toe.
Requestvalidatie en domeinautorisatie gaan aan de gesloten toestand vooraf;
onjuiste en domeinvreemde verzoeken blijven hun eigen fout krijgen. Geldige
GET/LIST/PUT/DELETE/RENAME-verzoeken in beide domeinen krijgen `Unknown`.

De vaste proef telt onafhankelijk nagerekend één bevestigde PUT, één onzekere
RENAME, tien latere Unknown-responses, tien domeinweigeringen en twee ongeldige
verzoeken. Dat correspondeert met ACKs 1, Unknown-responses 11, afgewezen
verzoeken zonder nieuwe I/O 22 en requestaantallen `[13, 11]`. De levende
toestand blijft sequence/revisie 1. Het schijforakel onderscheidt die toestand
van een al geschreven tweede record en controleert koude recovery op dezelfde
daadwerkelijke image.

Bij een ongeldige descriptor faalt de bestaande descriptorcontrole vóór enige
DMA-publicatie; de policy keert terug naar Ready. Bij reset-time-out blijft de
pool kernelbezit en gepind, is de policy Quarantined en wordt het device tijdens
de afsluitende cleanup niet opnieuw gereset of geïnitialiseerd. Een mislukte
herinitialisatie zet nu ook de policy zelf in quarantaine.

De acht foutcases blijven afzonderlijke testfeatures. De service-/wire-ABI,
capabilityrechten en de zelfstandige M6b-route worden niet verruimd. De normale,
recovery- en externe-killroutes mogen geen onverwachte Unknown-/faultmarkers
bevatten. De uitgebreide gate vereist 35 groepen en 31 verschillende echte
VM-invocaties; deze aantallen zijn coverage-eisen, geen door deze bronreview
bewezen uitvoeringsresultaten.

## Bevindingen en herstel

| ID | Bevinding | Herstel en controle |
|---|---|---|
| R08-01 | De nieuwe vrije capturedestination accepteerde `docs/verification` zelf. `_replace_archive` zou daarmee de gehele verzameling historische bewijsarchieven vervangen. | `archive_destination` accepteert uitsluitend één directe ondermap en weigert root-, escape- en geneste paden voordat een capture begint. Gerichte regressie geslaagd. |
| R08-02 | De nieuwe fault-oracle controleerde sequence/object/revisie/operatie maar niet de exacte request-ID. Een opnieuw berekende checksum kon een verkeerde requestidentiteit verbergen. | De faultprefix bindt nu PUT-request 10 en RENAME-request 11. Een gerichte test vervangt de ID door 1, 12 en `u64::MAX`, berekent geldige checksums opnieuw en verifieert afwijzing. |

## Werkelijk door deze reviewer uitgevoerd

Twee gerichte Python-hosttests via `python3 -B -m unittest`:

1. `test_m6c_uncertainty.FaultDiskOracle.test_valid_checksums_cannot_hide_wrong_request_identity`.
2. `test_m6c_uncertainty.ArchiveBoundary.test_verification_root_and_escape_cannot_be_replaced`.

Resultaat: **2 tests geslaagd, exit 0**. Exacte argv, werkmap, omgevingsaanpassing,
stdout/stderr en bronhashes vóór/na staan in `r08_review_checks.json`.
Deze tests gebruiken afzonderlijke tijdelijke hostfixtures. Zij zijn geen
nieuwe VM-runs en worden niet bij het aantal native runs opgeteld.

## Bewijsgrenzen en resterend werk

Foutinjecties 10/20 wijzigen een lokale completion-snapshot nadat het echte
virtio-device de used-ring heeft bijgewerkt. Time-outcases onderdrukken notify
en houden completionobservatie tegen tot de echte brokerdeadline. De
reset-time-outcase maskeert resetbevestiging. Dit toetst native foutafhandeling,
maar bewijst geen fysieke schijffout, fysieke stroomuitval of kwaadaardige-DMA-
containment. Een suppressed notify bewijst niet dat er niet is geschreven.

De dienstinstantie blijft na onzekerheid gesloten totdat een nieuwe instantie
herstel uitvoert; de voorgenomen gate toetst dat via een koude herstart. Geen
automatische serviceherstart of probleemloos doorgaan met documentbewerkingen
claimen. Beide appprocessen blijven volgens de proef wel ingepland worden.
De bestaande lokale adapterafbreking bij looplimiet of een misvormd succesvol
ABI-antwoord valt buiten de nieuw geïnjecteerde syscallfoutgevallen.

Vóór definitieve acceptatie: bouw en voer de volledige uitgebreide native gate,
bijbehorende captures en relevante regressie uit op bevroren bron; bind logs,
artifacthashes, werkelijke imagebytes en herstelparen aan die bron. Die
uitvoeringscontrole staat los van deze afgeronde bronreview.
