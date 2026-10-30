# PR #71 reviewreparaties

Beoordeelde bron: `3cf2fa9b5b51fdac84c9bd0eed964b5029606a59`. De reparatie voegt de Python-tests voor de replayoverdracht toe aan de bestaande native-probe-CI-taak voor host en target, maakt de negatieve repositorygrenstest onafhankelijk van een reeds bestaande targetmap en markeert oude client-CI-uitspraken expliciet als historisch. De bijbehorende README-bewijshash is bijgewerkt.

De oorspronkelijke test faalt met `FileNotFoundError` in een schone bronsnapshot (exit 1). Na de reparatie slagen alle negen tests (exit 0); zowel ervoor als erna ontbreekt `target/` en blijft geen repositoryfixture achter. De assertion controleert specifiek de fout voor de repositorygrens. `manifest.json` legt de exacte bron- en vervangingsblobs, commando's en exitcodes vast; de logs bewaren beide uitkomsten.

De bijbehorende merge behoudt main `c210366dd0f40c61c9f06f251bf525bebdbfbf90`, waarvan het verschil met `d791a342` bestaat uit 918 toegevoegde historische bewijsbestanden, zonder wijzigingen aan bestaande bestanden. Reeds aanwezig identiek bewijs blijft op grond van blobidentiteit behouden. Deze hosttests zijn geen nieuwe gastrun.

Exacte replay van handles uit de vorige boot vereist nog steeds de afzonderlijk beschreven native export, overdracht en startupintegratie. De compatibiliteitsnotitie beschrijft de ontbrekende binding in kandidaat-PR #96; dit bewijs claimt geen native NAS-, E2EE- of productacceptatie. De roadmap blijft 34/100.
