# TASK-USER-ALLOC — begrensde allocator voor gebruikersprocessen

Taakcontract, 30 oktober 2026, basis `cda42f2`. Eigenaar: `native_ui_apps`;
hoofdagent bezit SDK-/kernel-/appintegratie. Alleen `sdk/heap-allocator/**` en
dit document. Geen globale workspacewijziging of vaste heapgrowth-ABI-aanname.

## Gekozen engine en broncontrole

Hergebruik `linked_list_allocator = 0.10.6`, defaultfeatures uit. De meest recente
niet-yanked sparse-indexversie is gepubliceerd op 14 april 2026. Werkelijke
cratetarball en indexchecksum zijn vóór opname vergeleken:
`2b23ac50abb8261cb38c6e2a7192d3302e0836dac1628f6a93b82b4fad185897`.
De gepubliceerde manifesten verklaren Rust ≥1.61, Apache-2.0/MIT en `build=false`.
Zonder defaults zijn er geen transitieve runtime-dependencies; spin/nightly/
alloc_ref staan uit. De wrapper gebruikt de bestaande first-fit/coalescing-engine,
geen zelfgeschreven allocatiealgoritme.

Onderhouds-/veiligheidscontrole: upstream 0.10.6 repareert een mogelijke panic
bij een zeer grote Layout. De changelog noemt de oudere OOB-advisory
GHSA-xg8p-34w2-j49j, opgelost sinds 0.10.2, en de paddingreparatie van 0.10.4.
Dit is gerichte broncontrole, geen volledige audit of garantie dat alle fouten
zijn gevonden. De MIT-notice wordt met de package meegeleverd.

Bronnen: [upstream](https://github.com/rust-osdev/linked-list-allocator),
[crate-index](https://index.crates.io/li/nk/linked_list_allocator),
[gepinde tarball](https://static.crates.io/crates/linked_list_allocator/linked_list_allocator-0.10.6.crate),
[`Heap`-API](https://docs.rs/linked_list_allocator/0.10.6/linked_list_allocator/struct.Heap.html).
Bronnen en feitelijke versie gecontroleerd op 30 oktober 2026.

## Eigenaarschap en uitvoering

- Alleen exclusief aan dit proces toegewezen, aaneengesloten, permanent geldige
  RW+NX-geheugenruimte mag arena worden. BSS of bevestigde eigen heapgroei kan
  die ruimte leveren; een pointer/getal bewijst zelf geen kernelrecht of NX.
- `Arena::checked` controleert uitsluitend representatie, lengte, alignment en
  overflow zonder te derefereren. `initialize_raw` is unsafe: caller garandeert
  exclusief geldige `'static`-opslag. De veilige initializer neemt een exclusieve
  `'static mut`-bytebuffer over. Een mislukte shapecheck schrijft niets.
- Initialisatie gebeurt maximaal eenmaal. De engine krijgt vooraf genormaliseerde
  alignment en minimaal twee machinewoorden; ongeschikte/té kleine arena's en
  herinitialisatie geven fouten voordat allocatorstate of eerder werk verandert.
- Geen automatische groei, hostmalloc, virtueel-memorybackend of syscallfallback.
  De kernelagent werkt aan fallible eigen heapgroei; v1 gebruikt alleen een reeds
  bevestigde arena. Het ontwikkelquotum onder 8 MiB is geen productlimiet; de
  library legt geen dergelijke capaciteit vast.
- Het profiel is één userthread per proces, preëmptie toegestaan. Een atomaire
  atomaire lock beschermt engine/metrics; er zijn geen syscalls, yields, callbacks
  of logging onder die lock. Andere adresruimten gebruiken hun eigen instantie.
  Reëntrante allocatorcalls vanuit userinterrupts/signals zijn niet toegestaan.
- First-fit allocatie en vrijgeven kosten O(aantal vrije blokken). De vaste
  arenagrootte begrenst die lijst; dit is geen constante tijd of gemeten
  real-timegarantie. Eventuele grotere werksets vereisen metingen op het profiel.
- `GlobalAlloc` is unsafe volgens Rust: dealloc gebruikt exact de eerder
  geretourneerde pointer/Layout en precies eenmaal. De wrapper belooft geen
  detectie van willekeurige pointers/double-free en test zulke UB niet.

## Fouten en zichtbaarheid

Fallible allocatie onderscheidt niet-geïnitialiseerd, nul-/ongeldige aanvraag en
OOM. Alignment, extent en Layout-overflow worden vóór engineaanroep gecontroleerd.
Een conservatieve checked-arithmeticgrens voorkomt dat de engine bij extreme
hoge adressen/alignment een tijdelijk zoekadres laat rondlopen. Zulke aanvragen
geven OOM vóór een enginewijziging, ook wanneer een specifiek blok zou passen.
De gewone `GlobalAlloc`-API retourneert null bij falen; infallible Vec/Box-calls
kunnen daardoor het proces beëindigen. Userflows gebruiken `try_allocate` en
`Vec/String::try_reserve` voor zichtbare OOM zonder een onterechte OS-successtatus.
Mislukte reallocatie behoudt de oude geldige allocatie volgens het Rustcontract.

Usage/highwater betreffen uitsluitend de eigen arena: bruikbare capaciteit,
engine-gebruik/vrij, payloadgebruik, levende allocaties en totalen/mislukkingen.
Engine-gebruik telt het naar minimaal twee machinewoorden afgeronde blok; het
verschil met payload toont die overhead. Vrije-lijstheaders staan in vrije
blokken en zijn onderdeel van `engine_free`, geen extra externe heapruimte.
Totale vrije ruimte garandeert geen aaneengesloten groot blok. Historische
tellers satureren zichtbaar en lopen nooit rond. Geen geheimen worden gelogd;
deallocator wist niet automatisch alle plaintext of vrijgegeven pagina's.

De normale integratie installeert één `static HEAP: UserAllocator` als
`#[global_allocator]` en initialiseert deze vóór de eerste `alloc`-aanvraag.
Daarna werken gewone `alloc::Vec`, `alloc::boxed::Box` en `alloc::String` via
die instantie. `try_allocate(Layout)` geeft een rauwe, on-geïnitialiseerde
allocatie; de bijbehorende unsafe `deallocate(pointer, exact_dezelfde_layout)`
blijft verantwoordelijkheid van de caller. `usage()` kopieert alleen tellers
en geeft geen referentie naar de lock of de arena terug. Het gebruik van een
fallible API garandeert op zichzelf geen geheimenwissing of kernelrecht.

Rustcontract: [`GlobalAlloc`, Rust 1.90.0](https://github.com/rust-lang/rust/blob/1.90.0/library/core/src/alloc/global.rs).
Geen unwinding uit allocatorfuncties; `GlobalAlloc::alloc` ontvangt een geldige,
niet-lege Layout. Alleen de veilige wrapper biedt expliciet een `ZeroSize`-fout;
de tests roepen de unsafe globale API nooit met nulomvang aan. De standaard
`alloc_zeroed` en `realloc` van Rust verzorgen respectievelijk nulbytes en
behoud van de oude allocatie wanneer groei mislukt.

## Acceptatie en resterende native poort

Hosttests: initfouten zonder writes, alignment/prefix/suffix-canaries, extents,
klein/groot, hergebruik/coalescing/fragmentatie, OOM met behoud, counters en
werkelijke alloc::Vec/Box/String boven uitsluitend een eigen statische globale
testarena. Geen System/hostfallback in die collectionstest. Compileer engine en
wrapper daadwerkelijk scalar voor `x86_64-unknown-none` en leg bron-/artifact-
hashes vast. Dit bewijst geen native allocatie of dagelijkse browser-/AI-werking.

Native vervolg: root integreert een gewone userapp, verleent aantoonbaar eigen
RW+NX-arena's, controleert alloc/OOM/isolation en procescleanup in QEMU, en sluit
een eventuele heapgroeidescriptor pas na de afzonderlijke ABI-acceptatie aan.
Alle volledige productcriteria en het 8-GiB/32-GiB-testprofiel blijven behouden.

## Uitgevoerd bewijs

[`sdk/heap-allocator/verification/README.md`](../../sdk/heap-allocator/verification/README.md)
bevat reproduceerbare commando's, bron-/artifactbinding en het uitgevoerde
host-/crosscompilebewijs. Op deze bron slagen 15 gerichte tests, de afzonderlijke
Vec/Box/String-executable, Clippy en fmt. Wrapper en gepinde engine zijn scalar
gebouwd en hun daadwerkelijke objectcode is gecontroleerd. QEMU-, heapgroei-,
procescleanup- en isolatieacceptatie zijn nog niet uitgevoerd voor deze library;
de volledige release en de betrokken productcriteria blijven open.
