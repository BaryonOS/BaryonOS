# Visuele taal en rendertechnieken: van referentie naar Nieuw OS

**Onderzocht:** 3 november 2026, Europe/Amsterdam.
**Lokale bron:** `380111b68688c981eec6abb8fab29174a29f5601`.
**Bewijsniveau:** gelezen lokale en publieke broncode, officiële documentatie
en ontwerpinterpretatie. Geen nieuw host-/VM-renderresultaat, fysieke meting,
gebruikertest, onafhankelijke audit of releasevalidatie.

Ontwerpuitwerking:
[VISUAL-ARCHITECTURE-PLAN](../design/VISUAL-ARCHITECTURE-PLAN.md).

## 1. Belangrijkste technische les

Andere OS'en worden niet hoogwaardiger door één magische glasshader. De
samenhang ontstaat uit **kleurrollen, tekstmetrics, compositielaag, context,
input/focus, animatielifecycle en fallback**. Shaderwerk verbetert alleen het
onderdeel dat daadwerkelijk pixels combineert.

Voor Nieuw OS zijn bruikbare lessen:

| Referentie | Werkelijk mechanisme | Toepassing, zonder de desktop te kopiëren |
|---|---|---|
| KWin | Downsample/upsample-blur plus kleurtransformatie | Lokale tijdelijke systeem-/commandvlakken met gecontroleerd contrast |
| Mutter | Afzonderlijke achtergrondcontent en degradatiepaden | Stabiele omgeving los van persoonlijke appinhoud |
| GNOME Shell | Actor- versus achtergrondblur, aparte brightnessstap | Backdropafhankelijkheid en effectkwaliteit expliciet behandelen |
| Material Color Utilities | Perceptuele kleurrollen en scheme-parameters | Lokale optionele accentfamilie, niet overal dezelfde willekeurige hex |
| Flutter Impeller | Geschaalde filterdekking, radiusgrenzen en GPU-varianten | Eerst filter-/damagecontract, daarna shader |
| Windows Mica | Gedocumenteerde wallpaper-sampling, opaque materiaal | Statische contexttint kan zinvol zijn zonder live-blurkosten |
| Windows Acrylic | Gedocumenteerde transient-laag met opaque fallback | Effect ondersteunt context; overmatige stapeling vermijden |
| Apple Materials | Functionele bediening apart van content, legibilityvarianten | Transparantie mag; leesbare inhoud en systeeminterventies blijven duidelijk |

Dit zijn mechanisme- en ontwerpvergelijkingen, geen plan om KWin/Mutter,
Linux/XNU, Flutter of een bestaande OS-shell als Nieuw-OS-runtime te installeren.

## 2. Evolutie: wat veranderde er, en waarom?

Onderstaande tijdlijn is een compacte ontwerpinterpretatie, niet een volledig
archiefonderzoek van alle releases. Waar een oorspronkelijke historische bron
is gelezen staat een bron-ID. Zonder zo'n bron is de rij een analyse van het
bekende patroon, geen geverifieerde uitspraak over de interne motivatie van
alle ontwerpers of gemeten gebruikersvoorkeur.

| Periode / voorbeelden | Ontwerpprobleem en antwoord | Wat blijft bruikbaar | Wat slecht kan verouderen / Nieuw-OS-les |
|---|---|---|---|
| 2000-2006: XP/Luna, macOS Aqua, vroege glossy UI | Digitale bediening zichtbaar als fysieke knop/oppervlak; sterk herkenbare systeemfamilie | Duidelijke drukbare controls en actieve toestand | Glans op elk object maakt bediening en decoratie gelijk belangrijk |
| 2006-2011: Vista/Aero, Windows 7, Compiz/Ubuntu, compositing op Mac | Afzonderlijke appbuffers maken overlap, schaduw en vloeiende transities mogelijk | Venstergrenzen, ruimtelijk verband, stabiele backing stores | Transparante titelbalk op drukke wallpaper kan informatie verzwakken; effects zijn geen navigatiemodel |
| 2007-2013: vroege iOS/Android, skeuomorfisme | Touch heeft andere targets en minder pointerfeedback; fysieke metaforen verklaren sommige functies | Bekende gebaren, direct manipulation, herkenbare iconen | Leer/stof/kalenderdecoratie kan inhoud domineren en internationaal niet vanzelf spreken |
| 2010-2014: Metro/Windows 8, flat design, iOS 7 | Typografie, content en schermruimte krijgen meer gewicht; minder permanente ornamenten | Sterke hiërarchie, grote touchdoelen; iOS 7 gebruikt juist lagen, translucency en motion [S08] | Flat is niet automatisch helder; verborgen bediening of te lichte letters blijven risico |
| 2014-2020: Material, Windows 10/Fluent, moderner macOS, Adwaita/Plasma | Grote componentfamilies hebben een consistente grammatica nodig; diepte en motion koppelen handeling aan resultaat | Semantische rollen, herbruikbare componentstaten, voorspelbare animaties | Alle data als dezelfde card/pil tonen vermindert onderscheid; merken imiteren is geen systeemcoherentie |
| 2021-2026: Material You, Windows 11 Mica/Acrylic, actuele Apple Materials | Personalisatie en gelaagde bediening; materialen reageren op theme, context en toegankelijkheidsvoorkeur [S05,S06,S07,S09] | Afgeleide accentrollen, opaque fallback, content/controls scheiden | Wallpaperkleuren garanderen geen contrast; 'glas' wordt een trend wanneer de functie ontbreekt |
| Huidig GNOME/KDE/Ubuntu | Samenhang van toolkit, shell, symbolische iconen, licht/donker en compacte/dense bediening | Gedeelde style manager, componentfamilie, consistente focus [S01,S02,S10] | Linuxdesktop is een referentie, geen Nieuw-OS-kern of voorgeschreven layout |
| ChromeOS en spatial interfaces | Werk over desks bewaren; context en plaats uitbreiden zonder basisacties af te schaffen [S11,S12] | Hervatbare werkindelingen, minimum nodige immersie, stationair referentiekader | Eindeloos canvas of 3D is geen oplossing voor alles; hoofd-/handvermoeidheid en verdwalen tellen mee |

Skeuomorphism, glossy, flat, neumorphism en glassmorphism zijn niet gelijkwaardige
productarchitecturen. Neumorphism met twee zachte schaduwen op bijna dezelfde
achtergrond heeft bijvoorbeeld geen betrouwbare controlgrens zonder extra
contrastsignaal. Een niet-oplichtende neutrale rand kan nuttiger zijn.
Dat is een contrast-/affordanceanalyse, geen hier uitgevoerde gebruikersproef.

Responsive betekent hier dat dezelfde betekenis een passende compositie krijgt
bij minder ruimte. Context-aware betekent dat bediening het geselecteerde
onderwerp volgt, niet dat een systeem ongevraagd de interface herschrijft.
Geen trend uit bovenstaande rij bepaalt automatisch hoe 2035 eruitziet.

### Tien eigenschappen die modern kunnen aanvoelen

| Eigenschap | Onderbouwing / ontwerpvertaling |
|---|---|
| Content heeft duidelijke prioriteit | Apple scheidt functional layer en content; Blender geeft main region voorrang [S07,S13] |
| Consistente semantische kleurrollen | Material schemes en Adwaita-modi, niet component-eigen palettes [S03,S10] |
| Herkenbare interactieve grenzen | Contrast, focus en states, niet alleen subtiele tint [S10,S15] |
| Contextuele tools zonder informatieverlies | Blender koppelt settings/assetfilter aan actieve tool [S13] |
| Rustige maar werkelijke status | Android privacy-indicator; cars tonen actieve microfoon [S09,S14] |
| Lokaal begrijpelijke diepte | Transient materiaal bewaart achtergrondcontext [S06,S07] |
| Motion verklaart ruimte | iOS 7 legt lagen/motion vast; visionOS waarschuwt voor verlies van referentiekader [S08,S12] |
| Onderbreken en hervatten zijn zichtbaar | ChromeOS saved desks; cars hervatten taken [S11,S14] |
| Adaptatie behoudt bediening | GNOME licht/donker/HC en materiaal-fallbacks [S05,S06,S10] |
| Eigen, doorlopende compositie | Nieuw-OS-synthese: objectanker en toolrelatie in boot-to-recovery-familie; nog te toetsen |

### Tien eigenschappen die gedateerd kunnen aanvoelen

Ontwerphypotheses, niet de stelling dat iedereen oude UI lelijk vindt:

| Eigenschap | Waarom hier onwenselijk |
|---|---|
| Appmerk domineert boven gebruikerswerk | Staat haaks op de bestaande Materiaalkeuze |
| Iedere taak begint bij een verplicht appgrid | Directe materiaal-/commandroute verdwijnt |
| Dubbele permanente balken zonder afzonderlijke functie | Meer chrome, geen extra betekenis |
| Alle elementen dezelfde card met schaduw | Hiërarchie is niet meer te zien |
| Wallpaper/glow belangrijker dan leeswerk | Ornament krijgt informatieprioriteit |
| Flinterdunne grijze letters als 'premium' | Contrast en leesbaarheid worden opgeofferd |
| Hover-only sluit-/layoutbediening | Touch/keyboard en ontdekbaarheid ontbreken |
| Hele vensters live transparent | Afleiding en onvoorspelbaar contrast over grote leesvlakken |
| Bounce/zoom als standaard op elke actie | Bewegingslast zonder verklarend voordeel |
| Installer, shell en apps als losse stijlfamilies | De computer voelt niet als één onderhoudbaar product |

2030-2035: contentcontinuïteit, lokale betekenisvolle toestand, goede tekst,
multiview en beheersbare dichtheid zijn plausibele duurzaamheidsvragen.
Refractie, pillen, specifieke kleuren en animatiecurves zijn vervangbare stijl.
Dit is een **ontwerphypothese**, geen voorspelling.

## 3. Hoe het daadwerkelijk in andere code werkt

De publieke code is door het read-only VD-SOURCES-onderzoek gelezen.
Permalinks identificeren de bron, niet een licentie om code ongecontroleerd
over te nemen. In deze opdracht is geen externe code ingebouwd.

### S01 — KDE KWin: blur en kleurtransformatie

Organisatie: KDE; `blur.cpp`-revision
`cd18e4a1a620f671c3af9dc69e1df888a056c8be`.
De afzonderlijk gelezen shaderrevision is
`e618c04c9a9117c423f157da9c900921db5440c9`:
`shaders/downsample.frag`, `shaders/upsample.frag`.
Dit zijn gepinde mechanismevoorbeelden, geen hier gebouwde gecombineerde KWin-versie.

De blur gebruikt een downsample-/upsampleketen. Kleurtransformatie omvat
contrast/verzadiging via `colorTransformMatrix` en de onscreen-shader.
Dit zijn verschillende kwaliteitsvragen: verzachten van de achtergrond en
voldoende contrast voor de voorgrond. Een extra matrix is geen
WCAG-conformiteitsbewijs voor de uiteindelijke pixels.

**Les:** materiaal als semantisch effect met region-, kleur- en kwaliteitsbeleid;
geen globale blur over iedere app. Blur- en contrastparameters afzonderlijk
testen, samen met opaque/reduced-transparency. Geen gemeten kostprijs voor Nieuw OS.
Een noise/dither-pass kan banding bestrijden, maar is niet universeel verplicht
of bewijs dat alle artefacten weg zijn.

### S02 — GNOME Mutter: achtergrond los van appcontent

Organisatie: GNOME; contentrevision
`0d124ad8bb86161ce8fe1a2e42dc936abf20f27a`.
`MetaBackgroundContent`, achtergrondtexture(s),
kleur/gradient en vignette zijn afzonderlijke compositiefuncties.
De code beschrijft beperktere paden bij GLSL-/texturebeperkingen.
`MetaBackgroundActor` koppelt die achtergrond aan de scene.

**Les:** achtergrond is een eigen, beperkte bron; rijke compositie heeft een
eenvoudiger equivalent. 'Geen GPU' hoeft niet 'ander OS-design' te betekenen.
Vignette is voor Nieuw OS geen verplicht decoratief effect.
Oude hardware-/fixed-functioncommentaren zijn geen actuele performancegarantie
of opdracht om diezelfde API in Nieuw OS te gebruiken.

### S03 — Material Color Utilities: HCT en schemes

Organisatie: Google / Material Foundation, Apache-2.0.
HCT combineert hue/chroma uit CAM16 met tone uit CIE L*.
`DynamicScheme` heeft onder meer theme-/contrastparameters.

**Les:** accent -> tonale families -> semantische rol -> contrastcontrole.
HCT is niet 'haal vijf kleuren uit wallpaper en gebruik die overal'.
Gelezen HCT-revision `d28f86669c0bae54ebb54c710cc5caac85427979`;
scheme-revision `dd62982dfdefdfc9232ce8d76e122fb1c6d188ae`.
Extreme kleurcombinaties kunnen chroma verliezen om een uitvoerbare kleur
te vinden. Opacity, blur en composited achtergrond veranderen contrast:
een comment over toonafstand is geen algemene toegankelijkheidsgarantie.
Hoog contrast vereist ook geometrie, zichtbare focus en niet-kleurinformatie.

### S04 — Flutter Impeller: blur is ook geometrie en resourcebeleid

Organisatie: Flutter; bron
`impeller/entity/contents/filters/gaussian_blur_filter_contents.cc`,
revision `58ac1dadd69dfd2b873738edc5527400337771b8`.

`kMaxSigma` begrenst sigma; transform-schaal beïnvloedt filterdekking/padding.
De revision bevat een wijziging rond aparte GLES-downsample-shaders wegens
een apparaatcompilerprobleem. Dit toont dat shaderportabiliteit werkelijk
werk vereist, niet dat alle GPU's dezelfde implementatie aankunnen.

**Les:** cap op filterbudget; lees-/schrijfregio en schaal exact vastleggen;
geteste shadervarianten en duidelijk fallbackpad.
Een Gaussian-filter uit een host/appengine is geen Nieuw-OS GPU-driver.
Deze historische engine-revision wordt niet als nieuwste Flutter-release gepresenteerd.

### S05 — Microsoft Mica: geen synoniem voor live transparantie

Officiële design/API-documentatie, pagina gedateerd 22 juli 2026;
documentrevision `1b116d52a415fb84b45d26219181a4d25949a760`.
De documentatie noemt Mica opaque en beschrijft eenmalig samplen van wallpaper,
active/inactive gedrag en fallback naar solide kleur bij onder andere
transparantie uit, Battery Saver, beperkt apparaat en high contrast.

**Les:** een statische contexttint is technisch en visueel een andere keuze
dan doorlopende backdropblur. Dat kan in Nieuw OS functioneel helpen zonder
alle contentvlakken doorschijnend te maken.
Hier is de DWM-uitvoering niet gelezen; WinUI heeft wel publieke onderdelen.
Geen claim dat alle Microsoft UI-code gesloten is.

### S06 — Microsoft Acrylic: tijdelijk en begrensd

Officiële design/API-documentatie, 2 oktober 2026;
documentrevision `f202a4ffce146c5345a6424f4b7db838077ecc89`.
Onderscheidt background-/in-appmateriaal, beveelt transient surfaces aan,
waarschuwt voor meerdere gestapelde acryliclagen en accenttekst op glas.
Documenteert GPU-/energiekosten en solide fallback.

**Les:** menus en quick controls kunnen translucency gebruiken om hun
broncontext te behouden; lange tekst en grote permanente vlakken niet
automatisch. Geen universeel verbod op glas, wel een functie en meting.

### S07 — Apple Materials: function layer versus content

Officiële actuele HIG, geraadpleegd als leesbare `.md`-versie op 3 november 2026.
De gewone HTML-pagina bevatte alleen de JS-shell; die alleen is onvoldoende
bewijs voor de richtlijninhoud. De Markdownversie is wél gelezen.

Liquid Glass vormt volgens de richtlijn een aparte control-/navigatielaag.
Regular versus clear heeft verschillende leesbaarheids-/achtergrondvoorwaarden;
de tekst waarschuwt tegen Liquid Glass in de contentlaag en overmatig maatwerk.
Standard materials hebben eigen rollen voor inhoudsdifferentiatie.

**Les:** functionele glassurfaces mogen sterk worden onderzocht; ook Apple
bedoelt niet 'ieder document is een glazen paneel'. Geen kopie van Apple
geometrie, iconen of productarchitectuur nodig.
Core Animation/SwiftUI-materiaalimplementatie is hier niet als broncode gelezen.

### S16 — GNOME Shell: actorblur is iets anders dan backdropblur

Organisatie: GNOME; `src/shell-blur-effect.c`, revision
`51b5358b7185b5da7bc8869aaac78034a8ae0bd3`.
`ShellBlurEffect` onderscheidt `SHELL_BLUR_MODE_ACTOR` en
`SHELL_BLUR_MODE_BACKGROUND`. De bron waarschuwt dat background mode duur kan
zijn doordat veranderende inhoud onder de actor niet zomaar gecachet kan worden.
`brightness_glsl` is een afzonderlijke kleur-/brightnessstap.
`MIN_DOWNSCALE_SIZE` en `MAX_RADIUS` begrenzen interne downscale-/filterkeuzes;
dit is geen bewezen maximum voor iedere aangevraagde totale visuele blurradius.

**Les:** cachegeldigheid hangt van de bron af. Voor Nieuw OS zijn dirty regions,
backdropgeneraties en een afzonderlijk kwaliteitsbeleid nodig; alleen 'cache
de blur' opschrijven is niet genoeg. De C-pipeline is gelezen, niet de
afzonderlijke GLSL-blurkern en geen nieuwe GNOME-/Nieuw-OS-benchmark.

## 4. Referenties buiten de OS-shell

**Blender [S13]:** de main region blijft primair, toolsettings volgen de tool,
Adjust Last Operation verklaart wat zojuist gebeurde, asset shelf filtert op
actieve tool. Bruikbaar voor Materiaal als gereedschapsrelatie en expliciete
undo/aanpassingscontext. Niet kopiëren: alle verborgen/middle-mousebediening
als verplichte consumenten-OS-interactie.

**Android for Cars [S14]:** glanceable status, actieve microfoon zichtbaar,
onderbreken/hervatten en niet-urgente notificaties aanpassen aan taakdruk.
Bruikbaar voor kort leesbare systeemstatus en focusbeleid. Een desktop krijgt
geen rijbeperkingen of overal een drie-taplimiet.

**ChromeOS [S11]:** desks opslaan en heropenen laat zien dat hervatbare
werkcompositie niet hetzelfde is als een launcher. Nieuw OS wil daarbij ook
objectidentiteit en toegang behouden. Dat volgt niet vanzelf uit saved desks.

**visionOS [S12]:** kies minimum nodige immersie, blijf bij bekende controls
voor standaardtaken, behoud een stationair referentiekader en voorkom
overweldigende motion. Bruikbaar voor 2D-ruimtelijke continuïteit; geen headset,
3D-canvas, parallax of eye tracking als desktopvereiste toevoegen.

**Flutter [S04]:** creative/mobile/web-appengine als geometrische
filterreferentie, niet als bewezen geschikte no_std-runtime.
**Webtoegankelijkheid [S15]:** bruikbare meetbare contrast-/bedieningsvragen,
geen bewijs dat het native OS al een werkende screenreader heeft.

Gaming, smart-home, automotive hardware en professionele dashboards zijn
niet allemaal als afzonderlijke codebases onderzocht. Geen volledigheidsclaim
voor ieder voorbeeld uit de oude bijlagen; dit plan selecteert sterke
mechanismebronnen in plaats van oppervlakkige voorbeeldenlijsten.

## 5. Lokale code: wat is er werkelijk?

| Bron op `380111b6` | Gelezen eigenschap | Bewijsgrens |
|---|---|---|
| `ui/graphics/src/lib.rs` | `Canvas::new`, clipping, `blend_pixel`, afgeronde vormen, lijnen en iconen; `blend` met RGB/coverage | Scalar rendering in caller-owned slice; geen GPU-blur of rendergraph |
| `ui/graphics/src/theme.rs` | `MATERIAAL` paper/ink/iris; `wallpaper` doet één `fill_rect` | Paletvelden bewijzen geen gradient of illumination |
| `ui/graphics/README.md` | Open Sans 1.10-atlas, discrete maten, 100/200%; beperkte tekendekking | Geen algemene Unicode-shaping, bidi, IME of fontfallback |
| `ui/native/src/design.rs` | Compile-time CSS-bridge; hoofdkleuren uit graphics, andere waarden nog uit prototype | Gedeeltelijk hergebruik, geen volledige uniforme OS-tokenlaag |
| `ui/prototype/tokens.css` | Groen/petrol, system-ui, 4px-spacingbasis, CSS-shadow/motion | Browserwaarden, geen native implementatiebewijs |
| `sdk/examples/window-session/src/scene.rs` | Vertrouwde chrome, appnamen, topbar/dock; `render` opaque-bodyshortcut en `render_reference` | Appcentrische twee-venstersessie; geen onbeperkt ruimtelijk OS |
| `docs/implementation/NATIVE-WINDOW-SESSION.md` en ADR-053 | Privébacking stores, twee apps, 1.400.000 pixels; compositor enige DisplaySource | Gewijzigde productgeometrie vereist later eigen quota-/acceptatiecontract |
| `ui/native/src/desktop.rs` | Accounts/Login/Home/Search/Material; guest expliciet geen echte auth | Geen beveiligd account uit een loginformulier afleiden |

**Geen reden de eigen kernel te vervangen.** Vormgeving hoort in gebruikersruimte.
OSDev *Compositing*, W003354, revisie 28115 (9 juli 2023), verklaart
surface-/compositorrollen. De genoemde shared-memoryroute is leercontext,
geen normatief voorschrift; Nieuw OS heeft juist een eigen begrensd SUR1-pad.

### Belangrijkste renderimplicaties

Opaque content bewaart het eenvoudige kopie-/occlusionpad. Een live-blurlaag
maakt pixels onder het vlak opnieuw relevant; blur moet die achtergrond
binnen toegestane compositorprivébuffers lezen. Een app krijgt niet de
achtergrondpixels van andere apps terug.

Damage moet filterranden en backdropafhankelijkheden meenemen. Cachekeys
moeten inhoud/generatie, geometrie, schaal, thema en effectparameter omvatten.
Compositorherstel invalideert oude texture-/surfacegeneraties; een stale cache
is geen besparing maar een fout. Lock/aanmelding mogen geen oude persoonlijke
backdrop of thumbnail laten zien. Hiervoor is een echte sessiegrens nodig.

De bestaande RGB-coverageblend is niet automatisch lineair-licht of
premultiplied-alpha compositing. Nieuwe blend-/filterkwaliteit vergt een
afzonderlijk getest contract. High-DPI-schaal verandert ook halo, padding,
hitgebieden en damage, niet alleen fontgrootte.

### Rekenvoorbeeld, geen benchmark

1920 x 1080 x 4 bytes = **8.294.400 bytes** per volledige 32-bit-buffer.
Vier extra full-screenbuffers zijn circa **31,6 MiB**, exclusief apps, tiles,
textures en padding. Dat mag niet ongemerkt aan de huidige quota worden gehangen.
Een regionale filter van 512 x 320 met twee quarter-resolutionbuffers kan
veel kleiner zijn, maar precieze taps, damage en latency moeten worden gemeten.

Opaak -> statische tint -> lokaal cached effect -> GPU-filter is een
**kwaliteitsladder**: dezelfde functionele compositie, niet vier verschillende
productontwerpen. De gebruiker kan transparantie en motion beperken zonder
toegang, navigatie of informatie te verliezen.

## 6. Bronnenregister

Alle onderstaande bronnen zijn geraadpleegd op 3 november 2026.
Codepins zijn onderzoeksreferenties, geen dependencylockfile.
Niet-gepinde documentpagina's kunnen later veranderen.

| ID | Auteur/organisatie, titel/versie | Vindplaats |
|---|---|---|
| S01 | KDE, blur.cpp `cd18e4a1a620f671c3af9dc69e1df888a056c8be`; shaders `e618c04c9a9117c423f157da9c900921db5440c9` | [blur.cpp](https://invent.kde.org/plasma/kwin/-/blob/cd18e4a1a620f671c3af9dc69e1df888a056c8be/src/plugins/blur/blur.cpp), [downsample](https://invent.kde.org/plasma/kwin/-/blob/e618c04c9a9117c423f157da9c900921db5440c9/src/plugins/blur/shaders/downsample.frag), [upsample](https://invent.kde.org/plasma/kwin/-/blob/e618c04c9a9117c423f157da9c900921db5440c9/src/plugins/blur/shaders/upsample.frag) |
| S02 | GNOME, actor `d653914601996934fcd959ea56cd43132fa77a83`; content `0d124ad8bb86161ce8fe1a2e42dc936abf20f27a` | [actor](https://gitlab.gnome.org/GNOME/mutter/-/blob/d653914601996934fcd959ea56cd43132fa77a83/src/compositor/meta-background-actor.c), [content](https://gitlab.gnome.org/GNOME/mutter/-/blob/0d124ad8bb86161ce8fe1a2e42dc936abf20f27a/src/compositor/meta-background-content.c) |
| S03 | Material Foundation, HCT `d28f86669c0bae54ebb54c710cc5caac85427979`; scheme `dd62982dfdefdfc9232ce8d76e122fb1c6d188ae` | [HCT](https://github.com/material-foundation/material-color-utilities/blob/d28f86669c0bae54ebb54c710cc5caac85427979/typescript/hct/hct.ts), [DynamicScheme](https://github.com/material-foundation/material-color-utilities/blob/dd62982dfdefdfc9232ce8d76e122fb1c6d188ae/typescript/dynamiccolor/dynamic_scheme.ts) |
| S04 | Flutter, Impeller blur, revision `58ac1dadd69dfd2b873738edc5527400337771b8` | [Gaussian blur filter](https://github.com/flutter/engine/blob/58ac1dadd69dfd2b873738edc5527400337771b8/impeller/entity/contents/filters/gaussian_blur_filter_contents.cc) |
| S05 | Microsoft Learn, Mica, 22 juli 2026 | [Mica](https://learn.microsoft.com/en-us/windows/apps/design/style/mica) |
| S06 | Microsoft Learn, Acrylic, 2 oktober 2026 | [Acrylic](https://learn.microsoft.com/en-us/windows/apps/design/style/acrylic), [Materials](https://learn.microsoft.com/en-us/windows/apps/design/signature-experiences/materials) |
| S07 | Apple, HIG Materials, actuele Markdown-documentatie | [Materials](https://developer.apple.com/design/human-interface-guidelines/materials.md) |
| S08 | Apple Newsroom, Apple Unveils iOS 7, 10 juni 2013 | [Oorspronkelijk bericht](https://www.apple.com/newsroom/2013/06/10Apple-Unveils-iOS-7/) |
| S09 | Google, Android 12 Beta: Designed for you, 18 mei 2021 | [Android 12](https://blog.google/products-and-platforms/platforms/android/android-12-beta/) |
| S10 | GNOME, HIG UI Styling, actuele documentatie | [UI Styling](https://developer.gnome.org/hig/guidelines/ui-styling.html) |
| S11 | Google Chromebook Help, Set up & manage multiple desktops with desks | [Desks](https://support.google.com/chromebook/answer/9594869?hl=en) |
| S12 | Apple HIG, Designing for visionOS, changelog tot 2 februari 2024 | [visionOS](https://developer.apple.com/design/human-interface-guidelines/designing-for-visionos.md) |
| S13 | Blender Manual, Regions, actuele manual | [Regions](https://docs.blender.org/manual/en/latest/interface/window_system/regions.html) |
| S14 | Google, Design for Driving: Design Principles | [Cars](https://developer.android.com/design/ui/cars/guides/foundations/design-principles) |
| S15 | W3C, WCAG 2.2 Recommendation, actuele publicatie | [WCAG 2.2](https://www.w3.org/TR/WCAG22/) |
| S16 | GNOME Shell, ShellBlurEffect, `51b5358b7185b5da7bc8869aaac78034a8ae0bd3` | [shell-blur-effect.c](https://gitlab.gnome.org/GNOME/gnome-shell/-/blob/51b5358b7185b5da7bc8869aaac78034a8ae0bd3/src/shell-blur-effect.c) |

### Niet als bewijs gebruikt

De zoekpogingen voor enkele oude Windows-/Material-URL's gaven 404;
de Material 2-intropagina gaf alleen HTML/JS, geen leesbaar volledig ontwerpverhaal.
Die resultaten onderbouwen geen precieze historische claims. Het algemene
zoekresultaat werd evenmin als primaire bron gebruikt.

Apple/Microsoft-richtlijnen zeggen wat hun interface zou moeten doen;
ze bewijzen hier niet de gesloten compositoruitvoering.
Externe bugmeldingen over blur/contrast zijn niet in Nieuw OS gereproduceerd
en krijgen daarom geen algemene benchmark-/regressieclaim.

Bronlezen bewijst techniek en ontwerpintentie binnen die bronversie. Het bewijst
niet dat Nieuw OS die techniek heeft, sneller/mooier is, veilig is of klaar is
voor dagelijks gebruik.
