# Actieve milestone: beeld-crop naar PNG

**ACTIVE, nog geen volledige bestandsconsumer opgeleverd.**
TASK-PLATFORM-M7-IMAGE-CROP op mainbasis
`681f1137e411d6c6fe536bd2bf499c27b34832be`.
Scope uitsluitend deze nieuwe subtree plus eigen receipt; overgedragen
BMP/EXIF/viewer-consumers en General/Veld/K05 blijven onaangeraakt.

De implementatie is daadwerkelijk gestart: no_std crop-kern hergebruikt
`nieuw-os-image-codec::Image::rgba8_into` voor bestaande EXIF-oriëntatie.
Rechthoekcoördinaten gelden voor het getoonde beeld, niet voor ruwe JPEG-
samplevolgorde. RGBA8 en alpha blijven byte-exact. RGBA16 wordt expliciet
geweigerd in plaats van stil naar 8 bit te worden afgerond.
Geen clamp, resampling, matting, kleurcorrectie of bronmutatie.
Extra geheugenbudget omvat de volledige georiënteerde tijdelijke buffer
en het cropresultaat; de caller bezit daarnaast de oorspronkelijke Image.
Tests gebruiken benoemde synthetische fixtures; die zijn geen decoderbewijs.

Volgende actieve implementatiestap: echte PNG/JPEG-input via bestaande
decoder, expliciete keuze voor metadataverlies en RGBA-crop naar PNG met
onderhouden upstreamencoder, vervolgens onafhankelijk pixel-/chunkorakel.
Geen eigen PNG-engine toevoegen. `png` 0.18.1 is een onderzochte kandidaat,
nog **geen** toegevoegde dependency: de encoder is std-gebaseerd en zal
alleen achter een expliciete host-CLI-feature vallen. De no_std crop-kern
blijft gescheiden. Licentie, closure, onderhoud en build scripts moeten
eerst worden vastgelegd voordat die manifestwijziging plaatsvindt.

Deze ACTIVE-bron is geen volledige milestoneoverdracht of native
beeldbewerkerclaim. Geen nieuwe PR zonder OPEN-PR-LEASE.
