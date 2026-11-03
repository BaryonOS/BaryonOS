# Universal Professional Platform — implementatiestatus

**Peildatum:** 3 november 2026. Dit is het centrale statusdocument voor het
implementatiespoor uit de opdracht Universal Professional Platform. Het
vervangt geen bestaande releaseacceptatie, rubric, componentcontract of
actuele eigenaar/fileclaim. Een prototype, contractkern of hosttest is geen
native productbewijs.

| Mijlpaal | Status | Aanwezig | Nog nodig |
|---|---|---|---|
| M1 — capability/performance profiler | **PARTIAL** | `platform-profile-core` modelleert Render, Compute, Memory, Storage, Display, Media, AI en Power. Uit gevalideerde bootinvoer worden alleen UEFI ConventionalMemory-bytes en GOP-resolutie/stride/pixelindeling gevuld. Ontbrekende metingen zijn `Unknown`. | Kernel-/driver-/service-invoer voor CPU-features/topologie, opslag, GPU, refresh/HDR, media, AI en power; profiel aanbieden aan clients; metingen en Legacy/Modern/Advanced-acceptatie. Geen automatische hardwareklasse of performancebudget. |
| M2 — policy en tijdelijke runtime-rechten | **PARTIAL** | `permission-policy-core` bewaart in een bounded in-memory model exacte regels voor app, resource, purpose, context en activiteit. Canonieke encode/decode en voorwaarden voor actieve taak/foreground zijn hosttestbaar. Uitkomst verleent geen kernelcapability. | Geauthenticeerde duurzame settings-store, geverifieerde app-identiteit, trusted activity/session broker, echte kernelcapability-uitgifte/intrekking en native negatieve tests. `Once`/`This session`, prompt-UX en access history ontbreken. |
| M3 — native app/packageplatform | **PARTIAL** | `app-package-core` verifieert dual-signed single-image pakketten; `sdk/app-management` en catalogus/storecode leveren gedeeltelijke hostcontracten. | Duurzame bron-/catalogusregistratie, installertransactie, expliciete rechtenflow, native app-image-laad- en launchroute, updates/verwijdering en koude herstelproeven. Zie [APP-PACKAGES.md](APP-PACKAGES.md); controleer taak-/file-eigenaarschap vóór overlappend werk. |
| M4 — Terminal- en shellfundament | **PARTIAL** | `nieuw-os-terminal-core` biedt een begrensde UTF-8-commandoregelbewerker en argv-parser met quotes/escaping; onondersteunde shellsyntax wordt expliciet geweigerd. Hosttests en `x86_64-unknown-none`-compilatie slagen. | Native Terminal-app/rendering en invoer, een gedeeld proces-I/O-contract en koppeling aan geautoriseerde process launch ontbreken. Zie [Terminal-kernbewijs](../verification/terminal-core-20261103/README.md); dit is geen native shell of proces-I/O. |

## M1/M2-bewijs en grenzen

Broncode staat in `crates/platform-profile-core/` en
`crates/permission-policy-core/`; workspace-registratie en lockfile zijn
bijgewerkt. Exacte gerichte commando's, logs, hashes en de tijdelijke
hostlinkerbeperking staan in:

- [M1 hardwareprofielkern](../verification/platform-profile-core-20261103/README.md)
- [M1/M2 policy- en profieltests](../verification/permission-policy-core-20261103/README.md)

De policycodec detecteert geen storage-tampering of rollback en schrijft
niets naar een gebruikersvolume. Het policytype moet eerst door een
geauthenticeerde settings-store worden geladen; de huidige decodefunctie is
alleen een structurele parser. Evenmin controleert deze kern dat
`activity_active` of `foreground` van een vertrouwde broker afkomstig is.
Zonder die grenzen is `PermitsActivation` geen basis voor runtime-autorisatie.

De profiler maakt nog geen instellingen-, resource- of performancekeuze.
UEFI ConventionalMemory is geen totaal geïnstalleerd of vrij beschikbaar
geheugen; een GOP-modus is geen bewijs van GPU-acceleratie, refresh rate of
fysiek displaygedrag. Andere domeinen blijven onbekend totdat de aangewezen
driver-/service-eigenaar een begrensde bron levert.

De Terminal-kern ontleedt alleen tekst. Zij start geen proces, kent geen
commando-identiteit, voert geen variabele-/globexpansie uit, interpreteert
geen pipelines of omleiding en maakt geen IPC-endpoints/capabilities. De
bestaande SDK heeft wel capability-begrensde IPC en launch-API's, maar nog
geen gedeeld Terminal-stdin/stdout-protocol of native Terminal-app.

De roadmapmeter bleef **41 → 41/100**. Deze M4-code en hosttests sluiten geen
bestaand vast rubriccriterium en bewijzen geen native, VM-, fysieke,
menselijke of releaseacceptatie.
