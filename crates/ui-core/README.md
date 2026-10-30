# ui-core — grafische bootstrap

FR-U01 voegt aparte fysieke invoerbouwstenen toe: `Set1Keyboard`, `Ps2Mouse`,
`Cursor` en `InputRouter`. Zij vervangen de onderstaande G0-ASCII-API niet.
De router bewaart focusgeneraties en voorkomt dat vastgehouden toetsen,
modifiers of knoppen aan een nieuwe focus worden overgedragen. Dit is pure
`no_std`-logica; echte apparaat-I/O, capabilityafdwinging en native
compositoracceptatie blijven afzonderlijke stappen. Contract en reproduceerbare
controles: [NATIVE-INPUT.md](../../docs/implementation/NATIVE-INPUT.md).

Eigen dependencyvrije, safe Rust `no_std`-code. Geen pointers, firmware, MMIO,
allocator, bestandstoegang of appbevoegdheden. Dit is een softwarebouwsteen voor
een grafische OS-proef; geen compositor, desktop, opslaginterface of native M7.

`Geometry` controleert 32bpp RGB/BGR, maximaal 1920×1080, stride maximaal4096
pixels en een volledige buffer. `Canvas` tekent alleen in private RAM en laat
scanlinepadding en bytes buiten zijn gevalideerde slice staan. Signed rechthoeken
worden vóór tekenen geclipt. De oorspronkelijke 5×7-debugglyphs tonen kleine
letters als hoofdletters; niet opgenomen tekens worden `?`. Dit is geen volledige
Unicode-, font-, schaal- of toegankelijkheidsimplementatie.

`render_debug_row` gebruikt slechts `width*4` bytes (max7680). De hardwareadapter
moet een rij eerst in eigen RAM renderen en daarna met gepaste volatile writes
naar de afzonderlijk gevalideerde framebuffer kopiëren. Een RAM-slice naar MMIO
maken is geen toegestane vervanging. `draw_material_debug` is dezelfde scène op
een RAM-canvas. Beide tonen expliciet **GRAFISCHE BASIS - GEEN DESKTOP** en
**ALLEEN RAM - NIET OPGESLAGEN**. Er zijn geen fictieve bestandsacties of knoppen.

`Keyboard` decodeert de begrensde Set1-QEMU-proef: make/break, beide Shift/Ctrl/Alt,
CapsLock, E0-navigatie en PrintScreen; E1/Pause consumeert zes bytes zonder tekst.
De hostadapter controleert controllerstatus/pariteit/timeout/AUX en roept bij
verlies `reset()` aan. Geen USB-, locale-, invoermethode- of hardwareclaim.
`TextLine<N>` bewaart alleen ASCII32–126 in een vaste array; releases en Ctrl/Alt
voegen niets in. Vol geeft `Full`; Enter geeft `Submit` zonder opslag te claimen.

Hosttests controleren geometrie, overflowafwijzing, guards/padding, clipping,
glyphlandmarks, beperkte gewijzigde schermregio, modifier-/prefixreeksen,
tekstlimieten en willekeurige scanbytes. Zij bewijzen geen hardware-uitvoering.
