# Semantische venstergrenzen bij 100% en 200%

## Probleem en herstel

De semantische rootnode `Window` gebruikt al de fysieke viewportafmetingen.
Nogmaals schalen leverde bij 200% een venster van 1606 × 1206 op voor een
viewport van 803 × 603. Onderliggende rechthoeken zijn logisch en moeten wel
worden geschaald. De fix behoudt de exacte fysieke venstergrenzen, ook bij
oneven viewportafmetingen, en laat de bestaande schaalomzetting van
onderliggende nodes intact.

De bestaande raster-/semantiekregressie controleert nu de venster- en
editorgrenzen bij 100% en 200%. Met alleen de nieuwe assertions en de
oorspronkelijke renderer faalt de test; met de rendererfix slaagt zij.

## Eerste brongebonden controle: PR #68

Bron: PR #68-head `9d538c20cd5924fd9d72b0dd9963de2b9b906e48`, plus de twee
bronwijzigingen die met Git-blob en SHA-256 in [validation.json](validation.json)
staan. Deze resultaten zijn eerder gepubliceerd op
`d0f0d28a666ed91984b215f9ba7bca7ca39a7b08`.

De rode regressie gaf exit 101; na de fix gaf dezelfde regressie exit 0.
Alle 20 UI-hosttests, Clippy met geweigerde waarschuwingen en de
rustfmt-controle slaagden met Rust 1.90.0. Exacte commando's, bronblobs,
doorlooptijden en loghashes staan in `validation.json`. De oorspronkelijke
logs en bronbinding zijn ongewijzigd bewaard. Alleen de toelichtende tekst
van dit document en de JSON-beschrijvingen is vertaald naar het Nederlands.

## Herhaalde controle op de geïntegreerde bron

De controles zijn afzonderlijk herhaald op integratie-PR #96-head
`10bc807f94bbee3eb21403f1521b9ebec41b202a` plus dezelfde kleine UI-patch.
De patch is op de bestanden van die exacte bron toegepast; bestaande
integratiecode, tests en de runtime met `zeroize` blijven behouden.

Ook hier faalde de nieuwe regressie met de oorspronkelijke renderer
(exit 101; 1606 × 1206 tegenover 803 × 603). Na de fix slaagden dezelfde
regressie, alle 20 UI-hosttests, Clippy en rustfmt (alle exit 0).
[post96-validation.json](post96-validation.json) bindt deze herhaalde
controles aan hun eigen bronbestanden en logs met het voorvoegsel `post96-`.
De eerdere resultaten zijn niet naar deze nieuwe bron omgelabeld.

De bijbehorende README-correctie van PR #72 beschrijft de bestaande
`zeroize`-dependency correct. Zij wordt bytegetrouw overgenomen van
`65a961cc`; de runtimecode en de dependencyversie veranderen daardoor niet.

## Bewijsgrens

Dit is bewijs van hostregressiecontroles. Er is geen nieuwe uitvoering in een
gast, schermlezerintegratie, fysieke hardwaretest, menselijke
toegankelijkheidsacceptatie of aftekening van een productcriterium uitgevoerd.
GitHub-controles op de uiteindelijke commit blijven een afzonderlijke
mergevoorwaarde. De bestaande roadmap blijft 34/100.
