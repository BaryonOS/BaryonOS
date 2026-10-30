# APP-SOURCE-SNAPSHOT-1 - bronstaat exporteren en reconstrueren

30 oktober 2026; TASK-APP-SOURCE-SNAPSHOT-CONTRACT-1.
**ONTWERP voor afstemming, geen bestaande API of implementatievrijgave.**
Alleen dit document is nieuw. Geen verifier, issuer, backend, opslagencoding,
anker, capability of native herstel geleverd. Apps blijft 2/8, totaal 34%.

## 1. Bronnen, taakgrens en invariant

Dit ontwerp vult uitsluitend de private-registrygrens uit
[APP-SOURCE-RECOVERY, sectie 1][recovery] in. Bronnen zijn afzonderlijk gepind:

| Bron | Versie en betekenis |
|---|---|
| Apps-core | [`dce3b05aa358f4dbde6f2349b426217779b40052`][core]: werkelijke `SourceRegistry`, `SourceSlot`, `TrustAnchors`, `add/rotate/revoke/verify_executable` |
| SEC-herstelcontract | [`05784243d3d1ea273b588de82069596648fbde4d`][recovery]: actor, duurzame publicatie, Unknown, koude actualiteit en installerherautorisatie |
| Exclusieve toewijzing | [`82664b56a03dad5f2cdd64314234c406dd74a299`][queue]: alleen deze nieuwe docleaf; geen core-, backend- of frozen-PR-writes |

De toekomstige interface moet iedere door de bestaande core bereikbare staat
verliesloos kunnen exporteren en reconstrueren: dezelfde SourceIds, exacte
publieke sleutelparen, exacte generaties, active/revoked-status en tombstones.
Zij mag geen bestaande geldige staat door strengere cryptopolicy afwijzen.
Reconstructie geeft **geen bewijs** dat die staat authentiek, geautoriseerd
of actueel is. Alleen de toekomstige vertrouwde adapter mag een kandidaat
na de afzonderlijke herstelcontroles actief publiceren.

Dit is een gerichte API-afspraak, geen byte-ABI, definitieve architectuur-ADR
of toestemming de genoemde functies nu toe te voegen. Namen hieronder zijn
voorgestelde Rust-signatures voor latere ownerafstemming.

## 2. Bestaande semantiek die behouden moet blijven

De huidige core bevat maximaal `MAX_SOURCES = 8` slots. `add` gebruikt een
leeg slot, begint op generatie 1 en weigert een reeds aanwezige SourceId of
identiek volledig `TrustAnchors`-paar, ook in ingetrokken slots.
`rotate` vervangt het huidige paar en verhoogt de generatie checked.
`revoke` verhoogt de generatie checked en bewaart de bron met haar laatste
sleutelpaar als tombstone. Een ingetrokken bron kan niet opnieuw worden
geroteerd, ingetrokken of toegevoegd. Er is geen delete/compact/regrant.

`valid_anchors` gebruikt exact de bestaande parsechecks:
`EdKey::from_bytes(...).is_ok()` en
`EncodedVerifyingKey::<MlDsa65>::try_from(...).is_ok()`.
Dat is geen aanvullende key-strength-, nonzero- of onafhankelijke
cryptografische geldigheidsclaim. De snapshotinterface hergebruikt later
dezelfde interne helper; zij verzint geen tweede sleutelvalidator.

Dubbele keys betekent nu gelijkheid van het **hele paar**, niet dat ieder
afzonderlijk Ed25519- of ML-DSA-component uniek moet zijn. Een rotatie wist
het vorige paar uit die slotstaat; de registry is geen volledige keyhistorie.
Een vroeger gebruikt paar dat inmiddels nergens aanwezig is, wordt niet
achteraf verboden. Behoud dit beleid; strengere historische sleuteluitsluiting
vereist een afzonderlijk expliciet besluit en andere opgeslagen toestand.

`u32::MAX` is een geldige opgeslagen eindgeneratie. Een volgende mutatie
weigert met `GenerationExhausted`; export of reconstructie mag de bestaande
eindstaat niet afwijzen, ophogen, afkappen of terugzetten naar 1.

## 3. Voorgesteld logisch record en API-gedrag

Een logisch `SourceRecord` bevat uitsluitend:

| Veld | Betekenis |
|---|---|
| `id: SourceId` | Exact de 32 bronidentiteitsbytes, niet een afgeleide hash of displaynaam |
| `anchors: TrustAnchors` | Exact Ed25519 32 bytes en ML-DSA-65 1952 bytes; alleen publieke keys |
| `generation: u32` | Exacte bestaande brongeneratie, niet service-epoch of artifactgeneratie |
| `state: Active / Revoked` | Expliciete semantische toestand; Revoked blijft een bezet slot |

Geen `Empty`-record, private Rust-layout, discriminantbytes, padding,
pointer, secret, verified-flag of capability in deze uitwisseling.
Label/URL/license, gebruikersdomein, store-identiteit en duurzame
commit-/ankerbinding zijn geen bestaande `SourceSlot`-velden. Zij blijven
afzonderlijke geauthenticeerde adaptermetadata, gekoppeld aan deze records
volgens het SEC-contract; de core mag ze niet zelf verzinnen.

### Export

Voorgesteld: `SourceRegistry::export_snapshot(&self, out: &mut [SourceRecord])
-> Result<usize, SnapshotError>`.

- Exporteer alle bezette slots, inclusief tombstones, in bestaande slotvolgorde.
  Een immutable borrow houdt deze kopie consistent met een enkele corestaat;
  de dienst serialiseert hem met mutaties.
- Retourneer het precieze aantal records. Ongebruikte uitvoerelementen zijn
  niet onderdeel van de snapshot; geen lezen van ongeschreven staartbytes.
- Te kleine buffer geeft `BufferTooSmall` zonder uitvoer of registry gedeeltelijk
  te wijzigen. Eerst tellen/valideren, dan kopieren. Geen truncate-succes.
- Geen gegenereerde nieuwe IDs, sleutelparen, generaties of tombstoneverlies.
  Geen registratie-, signering-, I/O- of actorcontrole als bijwerking.
- Core-export is geen duurzame ACK. Dienstexport bij Unknown mag hoogstens
  expliciet diagnostische kandidaatdata opleveren, nooit een authoritative
  herstelbron of succesvolle productie-export.

### Reconstructie

Voorgesteld: `SourceRegistry::from_snapshot(records: &[SourceRecord])
-> Result<SourceRegistry, SnapshotError>`.

Deze functie bouwt een **private kandidaat**; zij vervangt geen bestaande
registry en verleent geen vertrouwen. De vertrouwde dienst houdt het resultaat
buiten haar actieve `verify_executable`-/installerpad tot publicatie is toegestaan.
De huidige core heeft zelf geen Unknown- of geauthenticeerde-providerstate:
een `Ok(SourceRegistry)` kan daarom nooit als trust-token worden behandeld.

Volledige prevalidatie is verplicht:

1. Aantal hoogstens `MAX_SOURCES`; nul records is structureel geldig voor
   een nooit ingerichte registry, maar geen toegestane rollbackfallback.
2. Elke SourceId heeft minstens een niet-nulbyte; elke keyset voldoet exact
   aan de bestaande `valid_anchors`-semantiek.
3. Geen dubbele SourceId en geen dubbel volledig sleutelpaar in de hele
   recordset, ongeacht active/revoked. Geen deduplicatie of last-record-wins.
4. Active heeft generatie ten minste 1; Revoked ten minste 2, omdat de
   bestaande `revoke` een bestaande generatie verhoogt. `u32::MAX` blijft
   voor beide geldig; onbekende states of niet-passende veldbreedtes worden
   al door de toekomstige decoder expliciet geweigerd.
5. Neem alle gevalideerde records in opgegeven volgorde over naar bezette
   slots; initialiseer alleen de overige slots als Empty.

Geen reconstructie via publiek `add` en herhaald `rotate/revoke` naspelen:
dat zou generaties/keys kunnen vervormen, tijdelijke verificatierechten
creeren en onnodig werk doen. Bouw binnen de latere core-implementatie uit
gevalideerde semantische velden; geen transmute, raw memory of layoutkopie.
Geen gedeeltelijk resultaat bij fout. De actieve registry en eventuele
uitvoerbuffer van export blijven ongewijzigd.

Voorgestelde concrete structurele fouten zijn `BufferTooSmall`, `Full`,
`InvalidSourceId`, `InvalidKeys`, `DuplicateSource`, `DuplicateKeys`,
`InvalidGeneration` en `InvalidState`. De precieze Rust-errorvorm blijft
ownerafstemming; geen silent defaults of succesvormige foutvervanging.
`Stale`, `Unknown`, `Unauthenticated`, `WrongDomain` en `Rollback` zijn
afzonderlijke adapter-/hersteluitkomsten, geen door deze core bewezen feiten.

## 4. Actualiteit, publicatie en bestaande actieve staat

Een structureel geldige snapshot blijft onbetrouwbare invoer. De adapter
controleert actor/bronbeheerdomein, storebinding, geauthenticeerde complete
commitinhoud en onafhankelijke actualiteit volgens [SEC secties 3-7][recovery].
Caller-generaties, service-epochs, hashes of een geslaagde parse vervangen
geen van die controles. Een gesigneerde oude lijst kan nog steeds rollback zijn.

Bij levende vervanging met een bekende vorige actieve staat gelden bovendien:

- Geen bestaande SourceId of tombstone mag verdwijnen; er is geen deletebeleid.
- Lagere generatie wordt geweigerd. Gelijke generatie vereist hetzelfde
  sleutelpaar en dezelfde status, ook als het totale snapshotlabel nieuwer is.
- Een actieve bron mag op hogere generatie een andere active-state hebben
  na rotatie of Revoked zijn na intrekking. Een identiek huidig paar op
  hogere generatie is niet automatisch ongeldig: tussenliggende rotaties
  kunnen naar dat paar zijn teruggekeerd.
- Een bestaande Revoked-record moet exact gelijk blijven. Een hogere
  generatie of heractivering van die tombstone bestaat niet in de huidige
  mutatiesemantiek. Nieuwe bronnen mogen alleen als door de buitenste
  autoriteit bevestigde toevoegingen verschijnen.
- Deze vergelijkingen bewijzen geen ontbrekende tussenliggende geschiedenis.
  Ook een hogere generatie vereist de volledige buitenste autoriteitscontrole.

Bij cold boot is de oude RAM-staat afwezig. Dat maakt een lege of oudere
snapshot niet acceptabel: de onafhankelijke actualiteits-/lineagebinding moet
ontbrekende bronnen, tombstones of rotaties detecteren. Ontbreekt dat bewijs,
dan blijft herstel Unknown/geblokkeerd; geen `SourceRegistry::new()` als
succesfallback en geen productie-install/update op een kandidaat.

Volledige kandidaat, adaptermetadata en geauthenticeerde commit moeten onder
een enkele dienstpublicatiegrens wisselen. Op fout blijft de vorige bekende
staat behouden; bij mogelijke duurzame publicatie blijft de betrokken scope
gesloten tot authoritative reconcile. Het exacte crash-/ankerprotocol is
nog open en wordt niet door deze in-memory API geleverd.

Een export is een kopie, geen lease op actuele autoriteit. Rotate/revoke na
export of na pakketverificatie vereist opnieuw autoriseren en serialiseren met
installcommit. Een oude `SourceVerifiedExecutable` mag geen generatiecheck
omzeilen. Bronrevoke blokkeert nieuwe verificatie/update, maar verwijdert
geen bestaande app/data en verandert haar afzonderlijke launchpolicy niet.

## 5. Verplichte latere gerichte tegenproeven

Dit zijn acceptatiegevallen voor een toekomstige implementatie, **niet
uitgevoerde tests** in deze docleaf.

| Geval | Vereist resultaat |
|---|---|
| Leeg, een bron, volle registry, gemengd active/revoked | Export/reconstruct/export behoudt alle records en volgorde exact; dezelfde latere `add/rotate/revoke`-uitkomsten |
| Generatie 1 active, 2 revoked, beide op `u32::MAX` | Geldige eindstaten blijven behouden; mutatie op max weigert zonder wijziging |
| Na meerdere rotaties terug naar eerder paar | Huidige valide staat blijft accepteerbaar; geen verzonnen historische keyban |
| Verschillende paren met een gedeeld keycomponent | Zelfde acceptatie als bestaande core; geen strengere componentuniciteit |
| Buffer nul/een te klein; output groter dan nodig | Fout zonder partial write; succes telt alleen geschreven records |
| Nul-ID, malformed keys, dubbel ID/paar incl tombstone | Expliciete fout; geen repareren, overslaan of dedupliceren |
| Active generatie 0; Revoked 0/1; onbekende state | Expliciete fout; geen automatische generatie-/statusconversie |
| Ongeldige laatste entry na valide prefix; te veel records | Geen gedeeltelijke kandidaat of actieve vervanging |
| Zelfde generatie met gewijzigde keys/status; ontbrekende tombstone | Adapter weigert; actieve snapshot onveranderd |
| Lagere generatie; Revoked naar Active of hogere Revoked-generatie | Adapter weigert volgens behouden mutatiepolicy |
| Valid parse maar verkeerde actor/domein/commit/anker | Geen publicatie of nieuwe pakketacceptatie |
| Authentieke oude lijst; koude start zonder actualiteitsbewijs | Rollback/Unknown, geen lege lijst of service-epoch als vervanger |
| Crash tijdens restore/publicatie; alleen prefix gelezen | Geen gedeeltelijke actieve registry, geen succes-ACK |
| Rotate/revoke tussen export/verificatie en installcommit | Herautorisatie; oude snapshot/token omzeilt actuele bronstaat niet |
| Bronrevoke met reeds geinstalleerde app | Nieuwe install/update weigert; bestaande afzonderlijke launchpolicy behouden |

## 6. Oplevering en nog open grenzen

Documentlezing en consistentiecontrole zijn geen compilatie, hosttest,
native uitvoering, duurzaam herstel of menselijke audit. Geen criterium
sluit door dit ontwerp; de centrale voortgangseigenaar houdt 34 -> 34.
De core-export/reconstructie, actorissuer, backend, rollbackanker, decoder,
authoritative adapter en install-/launchwiring blijven niet geimplementeerd
door deze leaf. Geen automatische vervolgclaim op die schrijfgebieden.

Voor implementatie zijn afzonderlijke ownerafstemming, codevrijgave,
negatieve tests hierboven en de voorgeschreven native/durable gates nodig.
Een scoped SEC-interfacelezing moet deze concrete docfreeze toetsen aan
het bestaande herstelcontract; zij vervangt geen onafhankelijke menselijke audit.
Rollback van alleen dit document verandert geen opslag of runtimegedrag.

[core]: https://github.com/simonvanieperen/nieuw-os/blob/dce3b05aa358f4dbde6f2349b426217779b40052/crates/app-package-core/src/lib.rs
[recovery]: https://github.com/simonvanieperen/nieuw-os/blob/05784243d3d1ea273b588de82069596648fbde4d/docs/contracts/APP-SOURCE-RECOVERY.md
[queue]: https://github.com/simonvanieperen/nieuw-os/blob/82664b56a03dad5f2cdd64314234c406dd74a299/docs/WORK-QUEUE.md
