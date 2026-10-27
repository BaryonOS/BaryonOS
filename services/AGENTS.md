# Regels voor systeemdiensten
Lees rootinstructies en beveiligingsmodel. Elke dienst krijgt expliciete taken, inputs, bevoegdheden, quota en herstelgedrag.
IPC-schema's versieerbaar; grenzen en herkomst altijd valideren. Geen impliciete globale toegang. Controleer confused-deputy-risico en delegatie.
Pas docs/LOCAL-PRIVACY-CONTRACT.md toe. NAS-sync uitsluitend naar expliciet gekoppelde eigen NAS op eigen LAN; geen cloudrelay/fallback. AI-inferentiedienst heeft geen netwerkrecht. Model-/updatedownloader krijgt geen persoonlijke data of algemene NAS-credentials. Netwerk standaard uit tenzij de dienst het nodig heeft. Logs lokaal en zonder secrets/persoonlijke inhoud als standaard.
Gebruik onderhouden cryptobibliotheken; maak geen eigen primitives/protocollen. Entropy, key lifecycle, rollbackbescherming en failure modes horen bij de integratie.
Test dat een crash begrensd blijft en herstel niet ongemerkt ruimere rechten geeft.
