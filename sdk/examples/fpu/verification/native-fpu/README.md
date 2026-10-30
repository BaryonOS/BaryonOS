# Bewijsgrens TASK-FPU

Deze compacte bundle hoort bij bronfingerprint
`501ee1d8e6aff5ffe3ed054fd7f516a97eb4b42e484b6abdbe7c879d7e4972b4`.
De volledige ruwe run en wegwerp-ESP staan lokaal onder
`build/fpu-final/run/native-fpu-e7e63edb25/`; kernel en loader zijn door SHA-256
gebonden in manifest en runrapport. De gate eist volledige markergrammatica,
werkelijke raw33, geen timeout, brononveranderlijkheid en exacte snapshotbytes.

Native resultaat: drie schone procesinstanties, twee gelijktijdige registerbanken,
46 geverifieerde loops, 49 kloklezingen, achttien geweigerde klokargumenten,
36 PIT-preëmpties, 155 echte FXSAVE64 en 155 echte FXRSTOR64, één echte
unsupported-AVX #UD, twee normale exits en één hergebruikt processlot. Alle
8192 contextbytes zijn na cleanup nul en 49263 vrije frames komen exact terug
op de onafhankelijk berekende baseline. Dit is geen volledige releaseclaim.

Een onafhankelijke reviewer bouwde dezelfde bron opnieuw en draaide een eigen
VM met dezelfde kernelhash: `independent-review.json`. De aantallen loops en
interrupts mogen verschillen; de strikte relaties en alle verplichte waarden
moeten in iedere run kloppen. De laatste FILD-instructie én het operandadres
verschillen tussen A/B (`kernel-state-audit.json`). FOP wordt vergeleken, maar de
proef claimt geen onderscheidbare FOP-waarden of volledige pending-exceptiondekking.

De vier oude FPU-uit-profielen zijn opnieuw gebouwd en native uitgevoerd op
exact dezelfde bronfingerprint: x87-weigering, XMM-weigering, AVX-weigering en de
volledige M3-proces-/preëmptie-/fairness-/cleanup-/DF-proef. Alle vier raw33 en
`passed=true`. De SDK heeft zeventien geslaagde hosttests; de tien hosttests voor
het nieuwe harnas controleren foutieve evidence en schrijfgrenzen, geen FPU-
implementatie. Beide apps hebben een geslaagde freestanding build, Clippy met
`-D warnings` en formatteringscontrole.

De gedeelde rootbuilder is eigendom van de integrator. In deze werkboom werd
voor de native bouw dezelfde hook aangeroepen: `build_users()` retourneert
ELF-env en rapport; `build(None, 'native-fpu', 'fpu-final')` bouwt werkelijk de
kernel/loader; de caller bindt het hookrapport als manifestveld `fpu`. De nieuwe
gate controleert vervolgens dat beide exacte ELF-bestanden daadwerkelijk in
het kernelartifact staan. Na rootintegratie moeten bouw en VM-poort opnieuw
worden uitgevoerd op de geïntegreerde commit; deze branchproef wordt niet als
nieuw bewijs voor een gewijzigde rootimage gepresenteerd.

Niet uitgevoerd: fysieke hardware, ontbrekende-FXSR-boot, alle x87/SIMD-
exceptiecombinaties en productprofiel-/enginevalidatie. De mislukte reserved-
MXCSR-proef is afzonderlijk bewaard in `../mxcsr-tcg-gap/`; geen #GP-claim.
