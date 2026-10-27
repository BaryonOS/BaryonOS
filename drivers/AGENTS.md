# Driverregels
Lees rootinstructies en hardware/securitydocumentatie. Schrijf voor één exact virtueel of fysiek device.
Behandel device-data, descriptors en lengtes als onbetrouwbaar. MMIO, DMA, geheugenvolgorde, interrupts, time-outs en resetpaden krijgen expliciete contracten. Gebruik alleen minimale mappings/rechten. Een userspacedriver met onbeperkte DMA is niet voldoende geïsoleerd.
Begin met virtuele apparaten. Geen raw hostdevices/hostdisks aan testguests koppelen. Vermeld welke garanties emulator-only zijn en welke IOMMU/hardwaresteun nodig hebben.
Test ontbrekend/niet-reagerend device, foute ringlengtes en herstart zonder lek van capabilities of buffers.
