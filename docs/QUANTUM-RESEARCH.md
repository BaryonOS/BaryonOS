# Quantum als expliciet onderzoeks- en architectuurspoor

> Update revisie 5: de eerste bouwkeuzes staan vast in [implementation/QUANTUM-SPEC.md](implementation/QUANTUM-SPEC.md). Gebruik dat document voor directe implementatie; dit bestand bewaart de ambitie en achtergrond voor vervolgfases.
Revisie 4, 26 oktober 2026. De oorspronkelijke ambitie omvat zowel quantumweerbaarheid als het kunnen inzetten van quantumcomputers. Dit spoor begint bij het ontwerp en verdwijnt niet achter een vrijblijvende 'ooit later'-vermelding. Het is nog niet geïmplementeerd.

## Drie afzonderlijke doelen
1. **Bescherming tegen quantumaanvallen.** Ontwerp de crypto-inventaris, algoritmevervanging, sleutelrotatie, updateverificatie en communicatie voor postkwantumcryptografie. Dit draait op gewone hardware. De bestaande beveiligingsnotitie blijft leidend voor protocollen, entropy en implementatiecontrole.
2. **Quantumprogramma's uitvoeren.** Ontwerp een lokale, geïsoleerde uitvoerdienst waarmee apps een quantumtaak kunnen aanbieden, eerst aan een lokale simulator en later aan geschikte eigen quantumhardware.
3. **Eigen OS op een hybride quantumcomputer.** Onderzoek hoe de eigen kernel en diensten de klassieke besturingslaag vormen van een systeem met klassieke processors en QPU's, inclusief scheduling, apparaatbeheer, gebruikersrechten en ontwikkelgereedschappen.

IBM beschrijft huidige systemen met klassieke runtime-servers en quantumprocessors. QIR onderzoekt interoperabele representaties van hybride programma's; OpenQASM beschrijft klassieke en quantumconstructies. Dat zijn referentiepunten, geen opgelegde grenzen voor het uiteindelijke product (IBM, z.d.; QIR Alliance, z.d.; OpenQASM Contributors, z.d.).

## Onderzoeksfasen
| Stap | Onderzoek/resultaat | Acceptatiebewijs |
|---|---|---|
| Q0 — vanaf M0 | Quantumdreigingen, cryptografische keten, uitvoeringsmodellen en interfacekeuzes in één korte ADR | Onderscheid tussen feiten, hypothesen en experimenten; geen impliciete cloudafhankelijkheid |
| Q1 — lokale prototypefase | Kleine klassieke simulator met begrensde quantumjobinterface | Referentiecircuits, outputverdelingen binnen vooraf gekozen tolerantie, annulering, quota en ongeldige invoer getest |
| Q2 — lokaal QPU-contract | Backendinterface voor capaciteiten, jobs, resultaten, foutmeldingen en resourcebeheer | Simulator/testbackend doorstaat dezelfde contracttests; nog geen hardwarecompatibiliteitsclaim |
| Q3 — hybride systeemport | Eigen OS op de klassieke host/controller met één concreet lokaal QPU-platform | Gedocumenteerde fysieke opstelling, drivers, timing-/controllergrenzen en echte uitgevoerde quantumtaak |
| Q4 — verder onderzoek | Alternatieve uitvoeringsmodellen, hybride scheduling en eventueel quantumfuncties binnen systeemdiensten | Afgebakend experiment met vergelijking; geen algemene versnellings- of haalbaarheidsbelofte |

Q0 is onderdeel van de eerste startopdracht, naast technische boot en productconcepten. Q1 mag aanvankelijk op de ontwikkelhost draaien om een interface te onderzoeken; label dat als hostprototype. Een port van dezelfde runtime naar het eigen OS is een afzonderlijk resultaat. Q2 kan zonder QPU worden ontworpen en getest; Q3 vereist daadwerkelijk toegang tot geschikte eigen hardware.

## Concrete ontwerpvragen
- Welke taakrepresentatie past: een afgebakende OpenQASM-subset, een QIR-profiel of een ander controleerbaar formaat? Onderzoek en pin een versie; geen onnodige nieuwe taal voor het eerste experiment.
- Hoe worden capaciteiten, qubitbehoefte, circuits, meetherhalingen, cancellation en resultaten beschreven?
- Welke taken zijn klassiek, welke quantum en hoe lopen gegevens en bevoegdheden tussen die delen?
- Welke timingkritische functies horen bij gespecialiseerde controllerelektronica? Een gewone OS-scheduler krijgt geen onbewezen realtimegarantie.
- Welke reset-/initialisatiegarantie bestaat tussen gebruikers en jobs? Onderzoek isolatie en eventueel beperkte exclusieve toewijzing; neem geen veilige deling aan.
- Wat zijn meetbare voordelen voor een concrete workload? Simulatieresultaten en theoretische quantumvoordelen bewijzen geen sneller desktop-OS.

## Beveiliging en privacy blijven hetzelfde contract
Alleen eigen lokaal beheerde hardware, rechtstreeks of binnen het expliciet toegestane eigen LAN; geen externe quantumcloud of relay. Het NAS-contract betreft opslag, niet een algemene machtiging om berekeningen uit te besteden. AI-inferentie blijft op het apparaat zelf. Een eventuele quantum-AI-workload moet aan datzelfde lokale apparaatcontract voldoen en krijgt geen stilzwijgende uitzondering.

De quantumdienst krijgt alleen de benodigde jobdata, apparaattoegang en begrensde middelen. Geen recht op alle documenten, NAS-opslag of internet. Vang parserfouten en hardware-uitval op. Onbetrouwbare drivers/firmware blijven onderdeel van de onderzochte vertrouwensgrens.

## Bewijsstatus en onderzoeksruimte
Een cryptografisch postkwantumalgoritme maakt de OS-kernel niet automatisch veilig. Een simulator die een circuit uitvoert bewijst geen fysieke QPU-ondersteuning of quantumversnelling. Een OS op de klassieke host van een quantumcomputer is een echte architectuurdoelstelling; noem het niet 'alle kernelcode draait op qubits'.

Onderzoek verdergaande vormen expliciet wanneer daarvoor een concreet uitvoeringsmodel bestaat. Wijs ideeën niet uitsluitend af omdat bestaande OS'en ze niet hebben; claim ze ook niet op basis van een naam of diagram. Het apparaat kan volledig bruikbaar blijven zonder QPU; de quantumlaag is een optionele systeemmodule.

## Bronnen (APA 7)
IBM. (z.d.). *IBM Quantum hardware and roadmap*. Geraadpleegd op 26 oktober 2026, van https://www.ibm.com/quantum/hardware

National Institute of Standards and Technology. (2024a). *Module-lattice-based key-encapsulation mechanism standard* (FIPS 203). https://doi.org/10.6028/NIST.FIPS.203

National Institute of Standards and Technology. (2024b). *Module-lattice-based digital signature standard* (FIPS 204). https://doi.org/10.6028/NIST.FIPS.204

OpenQASM Contributors. (z.d.). *OpenQASM live specification*. Geraadpleegd op 26 oktober 2026, van https://openqasm.com/

QIR Alliance. (z.d.). *QIR Alliance*. Geraadpleegd op 26 oktober 2026, van https://www.qir-alliance.org/

Deze roadmap is een eigen ontwerpvoorstel; de bronnen bewijzen niet dat dit toekomstige OS al hardware ondersteunt.
