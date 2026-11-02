# Geïntegreerde N01-acceptatie — 2 november 2026

N01 is geaccepteerd: echte afzonderlijke CPL3-netwerkdienst en twee IPC-apps,
TCP/UDP, malformed packets, begrensde buffers, sluiten en appuitval. Alleen de
dienst bezit NIC/entropy; een app kan geen algemeen netwerkrecht creëren.

Beide geïntegreerde QEMU-cases slagen met rawexit33, werkelijke frame-/driver-
aantallen, source/artifactbinding en gecontroleerde volledige cleanup. Zie
`n01-integrated.json` en de bytegetrouw opgeslagen twee serialtranscripten.
De normale run startte op ba117d2; aansluitend zijn uitsluitend auteurmetadata
van ongepubliceerde agentcommits gecorrigeerd. De tree is bytegelijk aan e6ccff3;
de andere run gebruikt die hash. Dit is expliciet in het rapport vastgelegd.
Latere wijzigingen betreffen grafische apps, hostorakels en Windowsbundeling.

De bestaande volledige dienst-/SDK-/peerhostproeven en de defecte-client- en
uitputtingsregressies staan daarnaast in `../n01-native-service-20261102/`.
Een tweede AI-review is geen menselijke beveiligingsaudit. Geen claim van
fysieke NIC-ondersteuning, dagelijks internet, DHCP of TLS; N02/N03/N04 blijven open.

Bij dit N01-checkpoint was U01 nog open; inmiddels is het afzonderlijk
geaccepteerd in [U01-ACCEPTANCE.md](U01-ACCEPTANCE.md). De eerste twee-app
native grafische smokeproef toonde echte
vensters en geen fake UI, maar duurt142,08s in deze TCG-omgeving. Die meting is
geen soepele-bedieningacceptatie. Focus/drag/compositorherstel en ongesneden
hostbeeldvergelijking worden afzonderlijk beproefd vóór U01-punten.
