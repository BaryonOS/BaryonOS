# R07: bronreview van de drie aangescherpte M6a-controles

28 oktober 2026. Alleen-lezen tweede AI-review van commit
`f60445b26d31733cc6aaf557ee08820b2295b0df`.
Bestand kernel/src/m2/m3/m6.rs, SHA256
`57a60248432451bc1b63b5744e032ae26cd9107f03f8037bbbb5af51ee5c957a`.

Geen blokkerende bevindingen in deze concrete diff:

- FULL bindt de vierde geldige PUT aan id5/gen4, vereist de exacte ENOSPC-reply
  en daadwerkelijke clientontvangst, en behoudt de nul-I/O-delta.
- Service-errors bindt ids100–114 aan hun werkelijke foutinhoud, controleert
  uniciteit/fasevolgorde en eist volledige request/send/receive-maskers.
- Unknown bewaart de eerste publicatiestand, weigert een volgende muterende
  syscall en eist één exact terminalantwoord bij send en receive, nul ACKs en
  de exacte WRITE/FLUSH-aantallen. Prepared-reject vereist WRITEpub0.
- De receive-hook volgt op succesvolle Engine-receive en gevalideerde
  gebruikerskopie. De controle vertrouwt niet alleen een userspace-teller.
- Geen nieuwe overlappende Runtime-borrow over reset/cut/cleanup gevonden.
  Servicecuts introduceren geen nieuwe Unknown-status; de bestaande
  gekopieerde DMA-pool gaat vóór de eindcleanup uit de Runtime-borrow.
- Geen verruiming van capabilities, pointers of DMA-adressen in de diff.

Dit is bronreview zonder eigen build/QEMU-uitvoering en geen menselijke
audit. De syscallassertie is testinstrumentatie; zij bewijst geen algemeen
productiebeleid voor onbetrouwbare diensten. Werkelijke QEMU-poort en
gecontroleerde tegenproeven worden apart vastgelegd.
