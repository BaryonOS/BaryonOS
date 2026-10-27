# Gerichte N01-review

## Scope
Alleen-lezen review door een afzonderlijke AI-agent op clientencryptie, opslag-/beheercapabilities, nonce-/quotaadministratie, revocation, writerlock, netpolicy en browsermodel. Geen menselijke audit, geen grafische browser- of netwerkpenetratietest.

## Gerepareerde bevindingen

1. **Noncebudget en parallel voorbereide uploads:** staging reserveert capaciteit; commit controleert de limiet opnieuw; rollback/cancel geeft gereserveerde ruimte vrij. Grens 8.191→8.192 en heropening getest.
2. **Tijdelijke ciphertextsnapshots:** eigen achtergebleven `.state-<32hex>.tmp` worden onder writerlock opgeruimd. Cleanupfout geeft geen succesvolle wisclaim; vreemde bestanden blijven intact.
3. **Writerlockrace:** exclusieve recoverylock serialiseert het herstellen van een dood PID-lock; twee openers kunnen niet elkaars nieuwe lock wissen. Gecontroleerde multi-processprobe en regressies bevestigen de grens.
4. **UI-beheer via internet:** datakoppeling maakt geen adminrol. Expliciete lokale rol en LAN-route zijn bij iedere beheermutatie vereist; WAN/mismatch/netwerkverlies trekt de rol en wachtende wisactie in.

## Laatste routewijziging
De serviceplanner selecteert alleen reeds geverifieerde, geautoriseerde facts voor exact de gevraagde peer. LAN krijgt voorrang, daarna eerder toegestaan direct internet. Geen nieuwe toestemming, relay of vervangende host. De UI-routeobservatie blijft expliciet een mock en kan geen koppeling, internetconsent of adminrol creëren. Twee nieuwe UI- en vier nieuwe serviceproeven zijn onafhankelijk herhaald.

## Hercontrole en grenzen
De integrator heeft de gezamenlijke definitieve suite uitgevoerd: 89/89, exit0, plus demo en syntaxiscontroles. De reviewer herhaalde gerichte regressies en een tijdelijke procesprobe; deze zijn geen extra unieke testtelling. Bij de laatste tekstaanpassing is alleen het backupadvies verduidelijkt, zonder veiligheidslogica te veranderen.

Geen resterende concrete blocker voor het **afgebakende hostprototype**. Blijvend schrijfbaar keyherstel, rotatie, multi-device, native TLS/netwerk, bestemming-scope voor algemene apps, volledige rollbackpreventie en fysieke wisgarantie zijn niet geleverd. De browser is geen beveiligingsgrens. Tijdelijke proefbeperkingen zijn geen gewenste productervaring; DESIGN-AMBITION.md maakt gebruiksgemak en veiligheid gezamenlijke gates.
