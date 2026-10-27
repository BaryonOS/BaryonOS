# Kopieer vanaf de volgende regel
Je bent mijn hoofdontwikkelaar voor een zelfstandig open-source besturingssysteem. Het vooronderzoek en de eerste ontwerpkeuzes zijn al uitgevoerd. **Ga nu implementeren; begin niet opnieuw met een algemene onderzoeks- of conceptronde.**

Lees AGENTS.md, docs/PROJECT-STATE.md en docs/implementation/START-CODING.md. Lees vervolgens docs/implementation/FIRST-BATCH.md en alleen de specificaties die bij de betreffende taak horen. De implementatiebesluiten zijn de huidige bouwbasis; oudere onderzoeksdocumenten zijn achtergrond. Expliciete gebruikersvereisten blijven leidend.

Vaste productgrenzen:
- Eigen kernel en eigen hoogwaardige productervaring; geen Linux-distributie of bestaande kernel als basis.
- OS-gegevens op apparaat of eigen NAS binnen eigen LAN. AI-inferentie uitsluitend op het apparaat, zonder cloudfallback. Externe apps via gescheiden rechten.
- Quantum: postkwantumweerbaarheid plus lokale quantumuitvoering en een toekomstige eigen OS-besturingslaag voor hybride quantumhardware.
- Open source, toegankelijk, configureerbaar en onafhankelijk van verplichte bigtechdiensten. Gebruik de gekozen productrichting; geen nieuwe conceptselectie nodig.

Voer nu de eerste batch uit:
1. Controleer de echte projectmap, Git-status en toepasselijke instructies. Behoud bestaande wijzigingen. Voer tools/preflight.py uit en maak de gekozen ontwikkelomgeving bruikbaar binnen de beschikbare bevoegdheden. Inspecteer wat er werkelijk is; stel geen fictieve testresultaten op.
2. Geef maximaal drie builders afgebakend werk: B01 eigen boot/loader/kernel, P02 het gekozen interactieve productprototype en Q01 de lokale quantumsimulator. Volg FIRST-BATCH.md voor exclusieve schrijfgebieden. Beheer gedeelde rootconfiguratie en tools zelf; coördineer de interfaceversies. B01 bezit de boot-protocol- en ELF-parsercrates.
3. Schrijf broncode en passende tests. Gebruik de concrete contracten, schemas, limieten, schermen en testgevallen uit de specificaties. Raadpleeg alleen gericht primaire documentatie wanneer een concrete API, fout of incompatibiliteit dat nodig maakt.
4. Voer build, emulatortest en prototype-/simulatortests daadwerkelijk uit. Als een tool of toestemming ontbreekt: noteer de precieze blokkade en rond onafhankelijk werk af. Vraag maximaal één gerichte vraag als een noodzakelijke keuze echt niet zelfstandig kan worden opgelost.
5. Integreer, laat de wijzigingen reviewen, herstel fouten en controleer de gecombineerde versie. Werk PROJECT-STATE.md bij met code, echte resultaten en resterende concrete blokkades.

Doe de lokale implementatie en tests zelfstandig. Geen publieke push/release, betaalde infrastructuur, echte disks, firmwareflash, NAS-wijzigingen of productiekeys zonder aparte opdracht. Gebruik synthetische gegevens. De eerste batch bevat nog geen productie-OS of hardware-QPU-ondersteuning.

Lever code en een kort verslag: wat werkt, hoe ik het start, welke tests echt draaiden, wat nog niet kon en de eerstvolgende implementatiestap. Stop niet na een plan of een nieuw pakket onderzoeksopdrachten.
