# NAS-RESUME-1 — begrensde clientqueue, hostmodel

`no_std`, allocatorvrij, `forbid(unsafe_code)`. Hergebruikt uitsluitend de
lokale `nieuw-os-nas-core`-interface, `Session`, `Upload`, revisions en errors.
Geen nieuwe identity-/wire-/crypto-ABI, transport of providerimplementatie.
Invoer is **reeds versleutelde opaque data**; dit model versleutelt niets.
De enige dependency is de bestaande lokale core v0.1.0 zonder buildscript.

## Invarianten en taakgrens

Vier pending edits met ieder maximaal de bestaande 1.024 ciphertextbytes;
dit zijn proefgrenzen, geen vaste productquota. Vol/duplicate/overflow weigert
de nieuwe edit expliciet. De bestaande edit wordt nooit vervangen. Conflicten
behouden lokale bytes, de serverversie en na expliciete observatie ook een
geverifieerde kopie van die remote ciphertext. `take` draagt data over aan de
caller en is geen stil verwijderen. `Debug` drukt geen ciphertext/identifiers af.

De queue geeft geen autoriteit. `prepare` opent **zelf de gekozen namespace**,
waarna bestaande core-authenticatie, epochbinding en CAS worden afgedwongen.
`commit` gebruikt het echte staged ticket. Een PID, route, boolean of
checkpoint verleent geen recht. `Principal` mag alleen uit vertrouwde ingress
komen; dit is dezelfde aanname als de core, geen toegevoegde authenticatie.
Een echte remote adapter moet de gekozen NAS cryptografisch identificeren.
De bij staging bewaarde caller voorkomt dat een verkeerd gerouteerde
commit/cancel het ticket wegneemt; deze vergelijking verleent zelf geen recht.

Offline queueing doet geen servicecalls. Een in-memory `Checkpoint` bevat
geen live Session/Upload. `restore` zet alle niet-afgeronde edits op
`NeedsReconcile`, ook wanneer de checkpoint vóór de dispatch is genomen.
`Unknown`, stale/revoke en conflicten geven geen automatische replay.
Caller-owned checkpoints zijn **geen persistent schema, durable-save ACK,
antirollbackanker of daadwerkelijke crashbestendige lokale opslag**.
Oude server-RAMtickets moeten bij verbindingsverlies door de bestaande
serverlifecycle worden opgeruimd; de hosthersteltests openen een nieuwe core.

`reconcile` is expliciet en read-only. Een opgegeven revisiehint wordt door
`Service::read` gevalideerd; caller/session/namespace/object blijven gebonden.
Onveranderde base kan daarna opnieuw worden geprobeerd; exact waargenomen
doelbytes leveren `Observed`, **niet** de oorspronkelijke commit-ACK.
Een andere versie bewaart beide kanten, zonder overschrijven of samenvoegen.
Een bewust gekozen nieuwe revisie moet door de gedeelde object-/Key Service
opnieuw onder die context worden versleuteld; oude AEAD-bytes worden niet
stil onder een andere revisie hergebruikt. `cancel` ruimt een nog niet
gepubliceerde staging op via de bestaande core, maar bewaart de lokale edit.
Lagere revisies blijven geblokkeerd. Cleanupfouten worden apart teruggegeven.
De core heeft nog geen guest `head/latest`-API: automatische revisiediscovery
wordt hier niet met een nieuwe wire of een onbetrouwbare status vervangen.

## Bewijs en open afhankelijkheden

Hosttests gebruiken echte core-CAS/authority/storage recovery en synthetische
opaque bytes. Crosscompile is librarybewijs, geen runtime in Nieuw OS.
P02/P03/S09 en FR-P03/P04/P05 blijven open. Geen E2EE, echte netwerkuitval,
LAN/internetovergang, native sync, apparaatkoppeling of productclaim.
De bevroren probe ce52 en het afzonderlijke schijforakel zijn niet gewijzigd.

Voor de volledige keten ontbreken hier nog: geauthenticeerd gedeeld transport
en peerbinding; Security's object/key-lifecycleinterface; duurzame lokale
queue/versieankering; remote latest-revisiondiscovery; daadwerkelijke
Materiaalbediening en native client/NAS-uitvoering. Providerkeuze blijft
expliciet in de gedeelde contracten; dit model kiest/verandert geen provider,
krijgt geen providercredentials/inhoudssleutels en heeft geen fallback.

Bronvaste afstemming met de eigenaars:

- NET `4a0ce99ff6b88adcd0e2c6be0571b30390d8d1c8` biedt alleen
  `KeyServiceRandom` en certificate-timebouwstenen, **geen** native handshake
  of TLS-peer-naar-NAS-principalinterface.
- Security `7f67471` biedt intern `RecordContext` en
  `Service::seal_record/unseal_record`, niet een geautoriseerde NAS-app-API.
  Mapping van NAS-namespace naar volume/lineage/epoch-authority en herstel/
  onafhankelijke rollbackankering is niet vastgesteld. De queue leidt deze
  rechten/context niet uit request-ID, uitnodiging of B02-fixture af.

Gerichte commando's (Rust 1.90; eigen `CARGO_TARGET_DIR`):

```sh
cargo test --offline --locked --manifest-path services/nas/native-client/Cargo.toml
cargo test --offline --locked --release --manifest-path services/nas/native-client/Cargo.toml
cargo clippy --offline --locked --manifest-path services/nas/native-client/Cargo.toml --all-targets -- -D warnings
cargo clippy --offline --locked --release --manifest-path services/nas/native-client/Cargo.toml --all-targets -- -D warnings
cargo fmt --manifest-path services/nas/native-client/Cargo.toml --all -- --check
cargo build --offline --locked --release --manifest-path services/nas/native-client/Cargo.toml --target x86_64-unknown-none
```
