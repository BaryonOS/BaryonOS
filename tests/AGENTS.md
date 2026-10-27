# Testregels
Lees rootinstructies en docs/TEST-STRATEGY.md. Koppel iedere test aan een requirement/invariant.
Bootlogs komen uit een echte emulatortest; scripts mogen geen verwachte succeslog fabriceren. Harness faalt op timeout, ontbrekende marker en onverwachte exitstatus.
Mocks bewijzen uitsluitend de gemockte laag. Geef niet-uitgevoerde checks expliciet aan.
Isoleer images, seeds en tijdelijke bestanden. Geen internetdoelen, persoonlijke data of echte disks. Vergelijk prestaties alleen bij vastgelegde omstandigheden.
Rapporteer fouten zonder tests te verzwakken om groen te krijgen.
