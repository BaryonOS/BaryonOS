# ACPI S5 parser core

## Doel en trustgrens

Deze dependencyvrije `no_std`-crate valideert begrensde ruwe FADT- en
DSDT-SDT-snapshots en haalt daaruit uitsluitend de S5-slaaptypen en de
legacy x86 PM1a/PM1b-controlregisteradressen. De publieke invoer is
`AcpiTableSnapshot` met twee `TableSnapshot`-waarden (exacte bytes plus het
door de kernel vastgestelde oorspronkelijke firmwareadres). Die metadata moet
afkomstig zijn uit de versiegebonden BootInfo-snapshot; de crate-constructor
is geen autorisatiebewijs. De kernel valideert onafhankelijk de
snapshotversie, extents, tabelsoorten, reservering, overlap, mappings,
SDT-signatures/checksums en FADT/DSDT-koppeling.

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
kernelbroker blijft eigenaar van snapshotvalidatie, de niet-delegeerbare
poweroff-capability en de later exact afgebakende QEMU-only write/flush-actie.

## Begrensde AML-subset

De parser herkent alleen een root-scope `Name(_S5_, Package(...))` met exact
twee integerconstanten (`Zero`, `One`, `Ones`, `Byte`, `Word`, `DWord` of
`QWord`-prefix); beide waarden moeten in drie bits passen. De naamsegmenteisen
zijn bewust strikter dan alle AML-varianten: ASCII hoofdletters, cijfers en
underscore. Root-relatieve en expliciet absolute `_S5_`-namen worden herkend.
Een tweede rootdefinitie is ambigu en wordt geweigerd.

Scope-, method- en extended namespacepakketobjecten worden alleen overgeslagen
met een gecontroleerde `PkgLength` en kunnen geen geneste `_S5_`-bytes tot
rootdefinitie promoveren. Naamobjecten buiten `_S5_` ondersteunen alleen
letterlijke dataobjecten, begrensde package/buffer/stringdata of een
NameString. Top-level Alias, External, Noop, Mutex, Event en OperationRegion
worden beperkt structureel overgeslagen; onbekende AML-termen, expressions,
malformed lengtes, ongeldige packages en ontbrekende/dubbele `_S5_` geven een
expliciete fout. Dit is geen generieke AML-interpreter. Een firmware-DSDT die
buiten de subset valt, wordt niet geaccepteerd; verbreed de subset alleen met
bewijs dat objectgrenzen en namespacecontext correct blijven.

## Gerichte tests en bewijsgrens

De unitfixtures dekken SDT-signature/lengte/checksum, bronadresbinding,
PM1-GAS/ranges/overlap, hardware-reduced mode, AML-constanten, ontbrekende,
dubbele en niet-root `_S5_`, foutieve `PkgLength`, verkeerde packagegrootte en
onbekende AML. Ze zijn hosttests op synthetische bytes, geen firmware- of
QEMU-bewijs. De release-integrator beheert BootInfo-integratie, onafhankelijke
kernelvalidatie, profielmanifest, native broker en de poweroff-proef. Er is
geen H05-aftekening en geen fysieke hardwareacceptatie.
