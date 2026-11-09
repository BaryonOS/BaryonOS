# ACPI S5 parser core

## Doel en trustgrens

Deze dependencyvrije `no_std`-crate valideert begrensde ruwe FADT- en
DSDT-SDT-snapshots en haalt daaruit uitsluitend de S5-slaaptypen en de
legacy x86 PM1a/PM1b-controlregisteradressen. De publieke invoer is
`AcpiTableSnapshot` met twee `TableSnapshot`-waarden (exacte bytes plus het
door de kernel vastgestelde oorspronkelijke firmwareadres). Die door de
caller aangeleverde metadata is geen autorisatiebewijs. In het opt-in
`native-acpi-s5`-profiel komt het FADT-adres uit `Discovery.fadt`;
`FirmwareReader` leest FADT en DSDT uitsluitend uit de gecontroleerde
UEFI ACPI reclaim/NVS-geheugenregio's. De parser valideert daarna
SDT-signatures, lengtes, checksums en de FADT/DSDT-koppeling. Deze
native route vereist geen vooraf gekopieerde versiegebonden BootInfo-
tabelsnapshot; de adres- en geheugenautorisatie ligt bij de kernel.

De parser controleert de tabelsignatures, exacte gedeclareerde lengte,
checksum, maximumomvang en binding van het door FADT aangewezen DSDT-adres.
De legacy PM1-lengte moet minimaal twee bytes zijn. De parser gebruikt een
aanwezige `X_PM1a/b_CNT` GAS alleen als die een 16-bit System-I/O-register met
bit-offset nul en toegestane access size beschrijft; anders weigert hij.
Legacy registeradressen worden alleen gebruikt als de overeenkomstige GAS-
adres nul/afwezig is. Adressen buiten het 16-bit I/O-bereik en overlappende
PM1a/PM1b-registers worden geweigerd. Hardware-reduced ACPI wordt niet
ondersteund.

`S5Plan` en `Pm1Setting` hebben private velden en geen publieke constructor.
Hun getters geven slechts parsed waarden terug; ze verlenen geen capability.
`apply_to` berekent uitsluitend een gemaskeerde 16-bit waarde op basis van een
reeds gelezen waarde. Er is geen port-I/O, `unsafe`, pointerinvoer,
firmwarecode-uitvoering, shutdownsyscall, kernelautorisatie of hostactie.
Een plan mag nooit via IPC als shutdownopdracht worden aangenomen. De
kernel blijft de bevoorrechte eigenaar van tabeltoegang en de opt-in
QEMU-only PM1-writeactie. Een dienstgestuurde poweroff-capability en
voorafgaand flushen van diensten zijn nog niet geïmplementeerd.

## Begrensde AML-subset

De parser herkent een root-scope `Name(_S5_, Package(...))` met twee
integerconstanten of de vier velden die QEMU gebruikt: twee slaaptypen en
twee verplichte nulconstanten als gereserveerde velden. Ondersteunde constanten
zijn `Zero`, `One`, `Ones`, `Byte`, `Word`, `DWord` en `QWord`; beide slaaptypen
moeten in drie bits passen. Andere aantallen en niet-nulle gereserveerde
velden worden geweigerd. De naamsegmenteisen
zijn bewust strikter dan alle AML-varianten: ASCII hoofdletters, cijfers en
underscore. Root-relatieve en expliciet absolute `_S5_`-namen worden herkend.
Een tweede rootdefinitie is ambigu en wordt geweigerd.

Een letterlijke `Scope(\)` wordt structureel doorlopen, met maximaal acht
geneste root-scopes en een vaste stack, zonder recursie. Alle objectgrenzen
blijven beperkt tot de huidige scope; na het scope-einde gaat controle van de
omringende termen door. Dubbele `_S5_`-definities in afzonderlijke root-scopes
worden eveneens geweigerd. Andere scopes, methoden en extended
namespacepakketobjecten worden alleen overgeslagen met een gecontroleerde
`PkgLength` en kunnen geen geneste `_S5_`-bytes tot geselecteerde rootdefinitie
promoveren. Dit resolveert geen algemene namespace, aliases, dynamische
definities of methode-effecten; de toekomstige kernelactie blijft daarom aan
het expliciete QEMU-profiel gebonden. Naamobjecten buiten `_S5_` ondersteunen alleen
letterlijke dataobjecten, begrensde package/buffer/stringdata of een
NameString. Top-level Alias, External, Noop, Mutex, Event en OperationRegion
worden beperkt structureel overgeslagen; onbekende AML-termen, expressions,
malformed lengtes, ongeldige packages en ontbrekende/dubbele `_S5_` geven een
expliciete fout. Dit is geen generieke AML-interpreter. Een firmware-DSDT die
buiten de subset valt, wordt niet geaccepteerd; verbreed de subset alleen met
bewijs dat objectgrenzen en namespacecontext correct blijven.

Voor meerbyte-`PkgLength` moet het ongebruikte bitpaar 4/5 in de leadbyte nul
zijn; de subset weigert encoderingen die die bits toch zetten in plaats van ze
stil te maskeren. Alle vier lengtebreedtes, afgeknotte vervolgbytes, te kleine
lengtevelden en pakketgrenzen buiten de tabel worden gericht getest. Afgeknotte
naam-, string-, scope-, method- en extended objecten worden ook na een al
gevonden `_S5_` geweigerd: een gevonden resultaat stopt de validatie niet.

De publieke primaire implementatiereferentie voor de lengtecodering is
ACPICA, tag `R2025_04_04`, `AcpiPsGetNextPackageLength` en
`AcpiPsGetNextPackageEnd` in
[`source/components/parser/psargs.c`](https://github.com/acpica/acpica/blob/R2025_04_04/source/components/parser/psargs.c).
Die bron bevestigt de leadbyteverdeling, maximaal drie vervolgbytes en de
lengtebasis bij het begin van het lengteveld. Het weigeren van de ongebruikte
bits is hier een expliciete strikte parserkeuze; ACPICA maskeert ze.
De UEFI ACPI 6.5 HTML-specificatie gaf bij controle HTTP 403. Geen broncode is
overgenomen en er is geen specificatieconformiteit van een volledige
AML-interpreter geclaimd.

QEMU `v10.2.1`, `hw/i386/acpi-build.c`, regels 1157-1184,
bouwt `_S5_` daadwerkelijk als vier-elementenpakket binnen `Scope(\)`.
De oudere twee-elementen/root-Name-fixture was dus geen passende QEMU-fixture.
Deze publieke primaire emulatorbron is gelezen en op SHA-256 vastgelegd:
`263bcea198131385448fb5827b6a30a5943313712b7caf99ca18343e07ea1b77`.
Bron: <https://github.com/qemu/qemu/blob/v10.2.1/hw/i386/acpi-build.c#L1157-L1184>.
De synthetische testfixture modelleert deze objectvorm. Daarnaast is de
volledige FADT/DSDT van een afzonderlijke QEMU/OVMF-firmwareguest via QMP
uitgelezen en op de host geparseerd: PM1a `0x604`, S5-type nul, geen PM1b.
Dit bewijst compatibiliteit met die concrete firmwarebytes, niet de eigen
loader/kerneloverdracht of guest-poweroff. Profiel, tabelhashes, commandotrace
en de expliciet uit te voeren externe-fixturetest staan in `verification/`.

## Gerichte tests en bewijsgrens

De unitfixtures dekken SDT-signature/lengte/checksum, bronadresbinding,
PM1-GAS/ranges/overlap, hardware-reduced mode, AML-constanten, ontbrekende,
dubbele en niet-root `_S5_`, foutieve `PkgLength`, verkeerde packagegrootte en
onbekende AML. De GAS-matrix muteert voor zowel PM1a als PM1b afzonderlijk de
address space, bit width, bit offset, access size en adresgrens terwijl het
legacy-adres geldig blijft: een aanwezige ongeldige extended beschrijving
wordt geweigerd, niet vervangen. Extended registeroverlap en een onjuiste
`X_DSDT`-binding worden eveneens geweigerd ondanks geldige legacyvelden.
De reguliere suite gebruikt synthetische hostfixtures. Een afzonderlijke,
standaard genegeerde test accepteert expliciet opgegeven externe
QEMU-firmwaretabellen en controleert ook mutaties daarvan; ontbrekende
fixturepaden of adressen laten die test falen, niet stil overslaan.
Die hosttest voert geen gastcode of shutdown uit.
Since 5 November 2026, the `native-acpi-s5` kernel profile has used this
crate for native power-off in QEMU. Evidence and limitations are documented in
[`docs/verification/h05-acpi-s5`](../../docs/verification/h05-acpi-s5/README.md).
Service- or user-initiated shutdown, H05 acceptance and physical-hardware
verification remain outstanding.
