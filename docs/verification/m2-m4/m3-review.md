# Gerichte M3-review — 27 oktober 2026

Broncheckpoint: `6802b17`. Reviewers R03 en R03V zijn andere AI-agents dan de kernelbuilder. Zij lazen de relevante implementatie; de hoofdagent voert de volledige samengestelde gate uit. Dit is geen menselijke audit of formeel bewijs.

De review vond en liet vóór de rootpoort deze concrete afwijkingen herstellen:

- Een cfg-fout plaatste NMI/DF/CPL0-terminalguards in de verkeerde assemblyvariant. De M2- en M3-paden zijn gescheiden; M2 wordt opnieuw gebouwd en gestart.
- De exitimplementatie bevatte voorwaarden van de testfixture (`current==A`, code0, counter≥8). Generieke beëindiging en testobserver zijn nu gescheiden. Een eigen probe test B exit7 bij counter1.
- De exitobserver vergeleek aanvankelijk met de opstartcounter. De baseline wordt nu direct vóór beëindiging genomen, zodat voortgang werkelijk daarna moet plaatsvinden.
- Usergekozen return-RSP/RIP konden assertions veroorzaken. De layoutvalidator begrenst de numerieke adressen en effectieve paginarechten; een ongeldige context beëindigt alleen dat proces. Een eigen probe zet RSP0 vóór int80.
- PID en interne frame-/tabelidentificatie waren beperkt tot u16. PID is nu monotone u64 zonder wrap; de begrensde ASID is afzonderlijk. Hosttests dekken de grens65535 en uitputting.
- Cleanup wordt nu per proces getoetst: vrije frames vóór + eigen userframes + vier kernelstackframes + eigen paginatabellen = vrije frames ná. Daarna mogen geen eigenaarsrestanten bestaan.
- Een lege runqueue stopte de kernel; alleen hlt toevoegen zou IRQ0 vanaf CPL0 alsnog als kernelfout behandelen. Het expliciete idlepad gebruikt veilige kernel-CR3/stack. Alleen IRQ0 met CS8 én idleflag krijgt een begrensd assembly-EOI/iretq-pad; NMI/DF en andere CPL0-exceptions blijven terminal. De afzonderlijke proef controleert acht echte wakeups en stabiele resources nadat beide processen zijn afgesloten.
- Debugregisters worden expliciet uitgeschakeld/geïnitialiseerd vóór de eerste ring-3-overdracht; zij worden daarna niet door processen gewijzigd.
- De ELF-parser controleert ook congruentie modulo p_align. User-p_paddr verleent geen fysieke adresautoriteit; de kernel kiest eigen frames.

Bewijscriteria zijn eveneens aangescherpt: KS04 volgt op echte CPL3-uitvoering en private patroon-/counterobservaties; KS07 controleert iedere selectie binnen de rondes. Een waargenomen int80-frame met DF1 is vereist naast de usercontrole DF0 na terugkeer. De kernel-UD-proef krijgt een specifieke vector6/error0/CS8-marker. Gemeten guestcycles omvatten cleanup/selectie, maar zijn geen realtimegarantie of bewijs dat geen timerinterrupt verloren kan gaan.

De reviewers meldden na de gerichte herstelcontrole geen resterende concrete blocker in deze gelezen paden. De gate-uitkomst wordt afzonderlijk opgeslagen; een reviewmelding telt niet als geslaagde QEMU-test. Sidechannels, DMA/IOMMU, SMP, fysieke hardware en onafhankelijke menselijke review vallen buiten dit bewijs.
