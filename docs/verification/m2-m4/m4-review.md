# Gerichte M4-review — 27 oktober 2026

R04C las de door een andere agent geschreven capability-core; R04K las de kernel-usercopy/syscallintegratie die hij zelf niet schreef. De hoofdagent beoordeelde interfaces en transactie-/copypaden en voert de samengevoegde eindpoort uit. Dit zijn AI-reviews, geen menselijke audit.

Gesloten bevindingen:

1. **Rechten vóór nieuwe rightsvelden.** `derive` kon een onbekend rechtenbit eerder melden dan het ontbrekende DERIVE-recht. De huidige volgorde controleert de lokale handle en het vereiste recht eerst; gecombineerde fouten hebben gerichte hosttests.
2. **Alle transfercaps vóór resources.** Bij twee caps kon de diepte8 van cap0 al EQUOTA geven terwijl cap1 een ongeldige handle had. Het tegenvoorbeeld faalde vóór herstel met Quota in plaats van BadHandle (exit101), en slaagt na herstel. Alle transferhandles/rechten/subsets worden nu gecontroleerd vóór de diepte-/node-/queueresources. De transactie liet ook vóór herstel geen gedeeltelijke mutatie achter.
3. **Geen observer-userread vóór autorisatie.** De syscallwrapper las bestaande receive-output voor testbewijs vóór de argumenten, size en endpointrechten waren afgehandeld. De copyplan-eigenaarscontrole voorkwam een vreemde-paginaread, maar de volgorde voldeed niet. De snapshot is verplaatst naar ná unusedargs/size/endpointauth en vóór de volledige outputvalidatie. Pre-authfouten beginnen/eindigen met `None` en lezen geen userbytes.
4. **Werkelijke mappingcapaciteit.** De bestaande mappingadministratie had64 plaatsen tegenover2048 in het contract. De builder heeft2048 plaatsen per proces naar een statische arena gebracht, met kleine Process-locals. Een extra user-ELF heeft35 imagepages +32 heap/stackpages =67; zijn tekst is bytegelijk aan de normale probe, de33 BSS-pagina's moeten werkelijk nul zijn. De complete gate krijgt deze regressie vóór M4.

De gelezen grens gebruikt integerpointers, checked-add, userbereik, effectieve U/S-/R/W-paginarechten en de actuele kernel-PID. Een send kopieert precies eenmaal naar een geïnitialiseerde kernelbuffer; de test-only snapshotrewrite wijzigt daarna alleen de oorspronkelijke userbuffer. Receive valideert iedere outputpagina vóór de Engine queue-/slotcommit; de daaropvolgende bytecopy kent in het één-CPU/één-threadprofiel geen user-unmap of gedeeld/DMA-framepad.

De copyproef is uitgebreid met geldige RO-input, misalignment, verboden pointers, cross-page/RO-output en volledige320-byte-readback na een vooraf met0xa5 gevulde output. Inputscratch is RO/NX; writable scratch weigert executable frames. Lifecycle trekt via Engine.exit autoriteit in vóór frames/tabellen worden vrijgegeven.

Corebewijs:21 hosttests,2.560 wire-bitmutaties,800 deterministische gemengde transacties en volledige Engine-equality bij fouten; alle hashes/commandexits zijn door root gecontroleerd. De kernelvingerafdruk is diagnostisch en wordt naast de bronreview, structurele invarianten, userbytes en echte ring-3-calls gebruikt. Hij is geen cryptografisch of formeel bewijs.

De reviewers vonden na het gerichte herstel geen resterende concrete blocker binnen hun gelezen scope. De samengevoegde rootpoort staat apart in `m4/suite.json` zodra daadwerkelijk voltooid. Global node-exhaustion en generationretirement hebben volledige hostgrensproeven; de relevante ring-3-quota zijn queue16,255 localslots en diepte8/9. SMP, DMA, fysieke hardware, algemene applicatie-/NAS-/netwerkrechten en een menselijke audit vallen buiten deze mijlpaal.
