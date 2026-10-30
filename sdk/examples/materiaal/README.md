# Materiaal: afzonderlijke native app

Echte no_std CPL3-caller voor de bestaande DocumentSession, typed INP1-client
en SUR1-client. Dit package bevat geen displaycap, raw PS/2-driver, blok-I/O,
hostopslag, netwerk, modelantwoord of vooraf ingevuld document. De compositor,
inputbron/policy en volumeprovider blijven aparte processen met eigen rechten.
Een testbackend in tests/ is alleen een protocoltegenpartij voor hostproeven.

De opstartpagina en kanaalverdeling staan in
[NATIVE-MATERIAAL.md](../../../docs/implementation/NATIVE-MATERIAAL.md).
Root integreert de kernelmanifesten en maakt de echte VM-testbeelden. Deze
bron of een geslaagd ELF bewijst op zichzelf geen native gebruikersketen.

## Bouwen en controleren

Met de projecttoolchain en centraal geïntegreerde SDK ABI1.8:

```sh
LOCALAPPDATA=/workspace/nieuw-os/.tools python3 -B sdk/examples/materiaal/verify.py
```

Het eigen gelockte workspace gebruikt exact de bestaande libraries, fontpin
en ontwerptokens. build.py bouwt alleen, wijzigt geen gedeelde bron of schijven
en geeft geen rechten. De linker levert vijf monotone LOAD-segmenten. R+NX
readonly consumeert ook fontpointertabellen vóór de writable wildcard; drie
64-KiB-buffers staan in exclusieve RW+NX-BSS, buiten de 64-KiB-stack. Geen heap
of hostmallocfallback. De echte kernel user::parse controleert het gebouwde
ELF; objdump/nm controleren de gemonomorfiseerde scalar code.

verify.py stopt als de centrale SDK nog ABI1.7 gebruikt. Voor de eerste lokale
ELF-proef was alleen de door root aangeleverde tijdelijke SDK1.8-patch actief;
die gedeelde files worden niet met dit package gecommit. De exacte gebruikte
SDK-bronhashes en patchhash blijven in verification/pre-native.json.

## Bediening in het geïntegreerde image

Materiaal leest de werkelijke volumelijst. Enter op Nieuw opent een leeg
document; typen bewerkt echte UTF-8-draftbytes. Ctrl+S start de streaming
opslag; Annuleren stuurt een echte ABORT zolang de upload loopt. Escape vraagt
terug naar Materiaal; bij gewijzigd werk zijn opslaan, doorwerken en expliciet
verwerpen beschikbaar. Tab/Shift+Tab wisselen tussen de zichtbare acties en
objecten. Opgeslagen objecten openen dezelfde bytes opnieuw via de provider.

De invoerclient ontvangt globale GOP-coördinaten. Alleen binnen het toegewezen
vlak worden de compositoroffsets afgetrokken. Daarbuiten eindigt een klik/drag;
terugkomen met een ingedrukte knop start geen nieuwe klik. Op100/200% gebruikt
de renderer dezelfde ontwerptokens, focus en tekstuele toestanden. Er is hier
nog geen gekoppelde schermlezer, volledige Unicode-invoer of multi-appsessie.

Per iteratie hoogstens vier PEEK/ACKs, één uploadstap en één tiletransactie.
Een oude halve paint mag een nieuwere draftgeneratie niet aftekenen. Ready
vereist een echte INP1-response en alle tile-ACKs van een huidig volledig frame.
Een verloren reply krijgt geen herhaalde mutatie of verzonnen ACK; opslag blijft
zo nodig onzeker en de draft blijft in het proces. Inputverlies trekt Ready in
en toont de werkelijke toestand; surfaceverlies stopt onzichtbare nieuwe acties.

Supervisor STOP5 wordt alleen op het vooraf verleende controlkanaal met de
verwachte initPID, exacte tag,1Bpayload en nul caps aanvaard. Dirty/saving geeft
STOP_REFUSED6, geen autosave/verwerpen/DONE. Alleen clean/idle plus expliciete
cleanup van de drie kanaaleigenaren geeft DONE3. Volledig sessieherstel, opnieuw
grants ontvangen en veilige persistente encrypted keys blijven afzonderlijk open.

De twintig hostketenproeven plus één echte ELFpolicytest staan in verification/.
De reviewfix heeft afzonderlijk rood/groenbewijs: Ready is uit tijdens iedere
nieuwe paint; PresentedGeneration verandert uitsluitend na alle actuele ACKs.
De gebouwde entry controleert RSPmod16==8 en springt rechtstreeks naar C.
QMP-bediening, daadwerkelijk gerenderde native schermen, koude heropening,
latency/stackmetingen en alle62 volledige releasecriteria blijven open.
