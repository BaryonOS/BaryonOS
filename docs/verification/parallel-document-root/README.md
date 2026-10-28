# Gecombineerde documentselectie en poortreview

De hoofdagent heeft de G11-library/probe en I13-poort samen met de centrale
buildselectie gelezen. Eén eigenaar gebruikt de twee bestaande endpoints;
LIST behoudt het concept, de geladen revisie en conflict/onzekerheidsgrenzen.
De opt-in verplaatst één GET en vervangt PUT/GET/LIST zonder de bestaande
kernelobserver of externe disk-oracle te wijzigen. De poort bindt de werkelijk
gecompileerde libraries en app, geen losse succesmarker.

Alle 27 gerichte Pythonmethoden op de samengevoegde werkbron slagen. De
werkelijke opdracht, exitstatus, ruwe streams en inputhashes staan in
`host-checks.json`. Daarnaast heeft de hoofdagent de volledige SDK-workspace
getest: 14 catalogusclient-, 14 documentsessie- en 13 runtimetests slagen.
De onafhankelijke ruwe SDK-commandobundel op dezelfde Rustinputs staat in
`../parallel-g11/`. De nieuwe LIST-tests zijn daarin afzonderlijk zichtbaar.

De default SDK-app heeft een andere ELF-hash dan de eerder geaccepteerde
G10-app. Daarom volgt behalve de nieuwe DocumentSession-poort een echte
normal/cold-controle van het default-SDK-profiel op de gezamenlijke freeze.
Er wordt geen byte-identieke default of reeds geslaagde nieuwe VM geclaimd.
Dit verslag is bronreview en hostbewijs; native uitvoering krijgt eigen logs.

## I15 hercontrole

Na de twee concrete G12-bevindingen zijn alle 30 gezamenlijke selectie- en
harnasmethoden opnieuw op de geïntegreerde bron uitgevoerd. Exacte opdracht,
exit 0, ruwe streams en bronhashes staan in `after-i15.json`. De oorspronkelijke
27-testuitvoering blijft afzonderlijk bewaard. De echte gepinde rustc accepteert
ook de aangehechte vorm `-Cno-redzone=no`; `attached-codegen-probe.json` bewaart
die concrete inventarisproef. I15 telt ook deze vorm mee en weigert een extra
of conflicterende ABI-instelling. Er is nog geen VM in dit bewijs opgenomen.

De hoofdagent heeft de B11-parserlengtes, slicegrenzen, checksumaccumulator,
private resultaatvelden, padding en afwezigheid van I/O/rechtentoekenning
gelezen. Alle vijf parserbestanden zijn bytegelijk aan de geteste B11-commit;
ook de 18 parsertests slagen op de samengevoegde checkout. `net-checks.json`
bewaart opdracht, exit, ruwe streams en de werkelijke inputhashes. De standaard
UDP-checksum- en Ethernet-paddingsemantiek is aan RFC 768 en RFC 894 getoetst.
Dit is een beperkte parserreview, geen bewijs van een volledige netwerkstack.
