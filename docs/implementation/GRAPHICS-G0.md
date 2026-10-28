# G0 — eerste native grafische basis

28 oktober 2026, vervolg op `7b4945f`. Dit contract gaat vooraf aan uitvoering.
G0 is een opt-in hardware-/invoerproef, geen volledige M7-desktop of compositor.
De bestaande native opslag en het browserprototype blijven afzonderlijke sporen.

## Resultaat en invariant

Een eigen kernel draait na ExitBootServices op zijn eigen M2-pagetables en tekent
via de door UEFI aangewezen framebuffer. Een begrensde PS/2-decoder verwerkt
werkelijke QEMU-toetsen tot een lokale RAM-regel. Het scherm vermeldt expliciet
dat dit de grafische basis is en nog geen desktop. Geen gesimuleerde bestanden,
opslagbevestiging, apps, NAS of AI in deze proef.

- Standaardboot behoudt het exacte BootInfo 1.1-contract. Alleen gekoppelde
  `graphics-g0`-features in loader en kernel gebruiken opt-in BootInfo 1.2:
  header 128 bytes, flags 15 en een vaste gevalideerde descriptor vanaf byte
  128 binnen dezelfde behouden BootInfo-pagina. Oude profielen krijgen geen GOP.
- De loader configureert/leest GOP vóór de laatste geheugenkaart/freeze; na
  EBS volgen geen GOP-, console- of andere firmwareoproepen.
- De kernel kopieert metadata vóór gebruik en controleert pixeltype, geometrie,
  stride, lengte, fysieke adresbreedte, overflow en overlap. Alleen RGB/BGR
  32-bit-pixels; geen blt-only- of willekeurige bitmaskerimplementatie.
- Framebufferpagina's krijgen uitsluitend een aparte, begrensde supervisor-
  RW/NX/UC-mapping. Geen verbreding van de virtio-MMIO-API en geen mapping van RAM
  of gereserveerde normale frames als framebuffer. PAT wordt gecontroleerd.
- `ui-core` is eigen safe Rust `no_std`, zonder heap of externe dependencies.
  Clipping en getypte pixelgeometrie begrenzen iedere schrijfpositie. Een
  private rijbuffer voorkomt een grote backbuffer; alleen zichtbare pixels
  gaan via volatile stores naar de framebuffer. Padding blijft intact.
- PS/2 Set1-invoer is een begrensde pollingproef op één QEMU-CPU, zonder IRQ1-
  aanpassing. Releases, modifiers en extended prefixes geven geen willekeurige
  tekst. De eerste tekstregel is begrensd ASCII; geen Unicode-/IME-claim.
- ESC beëindigt de test met een eigen G0-marker en raw-exit 33. Geen automatische
  groene test op basis van uitsluitend firmwarebeeld of een gemaakte screenshot.

## Acceptatie en grenzen

Hosttests controleren ongeldige framebuffergeometrie/overflow, clipping,
padding/canaries, kleurvolgorde, ongeldige scancodes, modifiers, backspace en
volle tekstregel. De native proef moet echte post-EBS/M2/G0-markers, geïnjecteerde
QMP-toetsaanslagen, framebufferbeelden vóór/na invoer en een correcte exit leveren.
QMP bestuurt uitsluitend het eigen QEMU-child zonder netwerk naar de guest.

Een zichtbaar getypte regel is geen opslag, geïsoleerde app, invoerbroker,
toegankelijke volledige shell of gebruikersdesktop. Die productintegratie
vereist daarna een afzonderlijke ring-3-display-/inputdienst en capabilitygrens.
De tijdelijke kernelbootstrap bevat uitsluitend begrensde diagnoseweergave;
object-, app- en opslagpolicy komen er niet in.

Eigenaar pure core/loader/bootmetadata/native adapter: G09. Root bezit
workspace/lockfiles, build-/QMP-harnas, documentatie en Git. Kernel-Cargo loopt
via K08, zodat één schrijver die gedeelde interface beheert. Rollback: aparte
checkout van `7b4945f`; bestaande bootprofielen blijven selecteerbaar.

Primaire bronnen: UEFI Forum, *UEFI Specification 2.11*, §12.9 Graphics Output
Protocol: https://uefi.org/specs/UEFI/2.11/12_Protocols_Console_Support.html;
Intel, *64 and IA-32 Architectures SDM*, pagetables/PAT volgens ADR-026.
Geraadpleegde bronversies, feitelijke tests en nog ontbrekende verificatie komen
in PROJECT-STATE.md. Projectlimieten zijn geen eisen uit de UEFI-specificatie.
