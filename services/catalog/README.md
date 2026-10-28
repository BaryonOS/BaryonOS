# Native documentnamen en catalogus (M6c)

`catalog-core` bevat allocatorvrije, dependencyvrije Rust `no_std`-logica voor
de eigen ring-3-documentdienst. De core verbiedt `unsafe`; de apart gebouwde
`user_service.rs` vormt de native IPC-/blok-I/O-grens.

Vier objecten, twee per endpointdomein, bewaren een UTF-8-label (maximaal 32
bytes) en maximaal 128 inhoudsbytes. Namen zijn labels; object-ID blijft
identiteit en autorisatiebasis. Gelijke namen mogen bestaan. GET/PUT/DELETE,
CAS-gecontroleerd RENAME en een atomische LIST van het eigen domein vormen
IPC v4 (224 bytes). LIST geeft alleen metadata, nooit inhoud of globale
activiteit; de catalogusrevisie telt alleen de eigen objectrevisies.

Journal v2 (`NOSM6C02`) bevat 32 append-only records van 512 bytes. Een RENAME
bewaart de bestaande inhoud exact, verhoogt de revisie en schrijft één record.
Recovery controleert predecessor, revisie en inhoudsbehoud. PUT/DELETE/RENAME
mogen pas na WRITE- én FLUSH-completion bevestigd worden. Het servicecontract
sluit na een onzekere mutatie alle geldige operaties, ook LIST, tot nieuwe
volledige recovery af; dit `Unknown`-gedrag is alleen via host-ABI getest.

Bekende native beperking, ook met `milestone-m6c`: de observer kan bij
WRITE-/FLUSH-fouten of time-outs panikeren vóór de client `Unknown` ontvangt.
Dit defect is niet gerepareerd; gecontroleerde native foutafhandeling is niet
gerealiseerd. Herstel vereist een afzonderlijke native reparatie met
WRITE-/FLUSH-fout- en time-outinjectie, controle van het werkelijk ontvangen
clientantwoord en nieuwe brongebonden native verificatie, zoals vastgelegd
in de [bewijsgrens](../../docs/implementation/M6C-NAMED-CATALOG.md#bekende-native-observerbeperking).

Deze core kan geen fysieke duurzaamheid waarnemen; het native adapterbewijs
is dus afzonderlijk nodig. FNV is diagnostiek, geen cryptografische authenticatie.
DELETE wist levende naam/inhoud, maar niet historische schijfbytes. Er is geen
ruimtehergebruik, encryptie, migrator, NAS of grafische desktop. Bestaande
M6b-v1-records worden expliciet geweigerd zonder formattering; de oude M6b-route
blijft afzonderlijk beschikbaar.

Exact contract en acceptatie:
[`M6C-NAMED-CATALOG.md`](../../docs/implementation/M6C-NAMED-CATALOG.md).
Uitgevoerde controles en nog open bewijs staan in `docs/PROJECT-STATE.md`;
deze README verklaart geen nog niet uitgevoerde test geslaagd.
