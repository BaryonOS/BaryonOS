# Mislukte causale netwerkrepro

Bron309ff53 plus uitsluitend bijgevoegde peerdiagnostiek; exacte v3-vingerafdruk staat in result.json. QEMUrun raw35, netgatefalse. Eerste frame86B, EtherType86dd: IPv6bron:: naar ff02::1:ff00:7, hop-by-hop/ICMPv6type131(MLDv1-membershipreport), MAC33:33:ff:00:00:07. Onze probe bouwt alleen ARP/IPv4; OVMFVirtioNetDxe/IP6initialisatie produceert dit prebootverkeer. Peer sluit fail-closed en daarna timeout de echteARPclient; geen OS-DMAcorruptie aangetoond.

Causale reparatie gebruikt de officiële EDK2-runtimeknoppen VirtioNetSupport,IPv4Support,IPv6Support metstring=no en romfile= op de intacte virtioPCI-NIC. Geen frames negeren, geen succescriteria verruimen. Native nameting volgt op eigen bevroren commit. Bronnen: [RUNTIME_CONFIG](https://github.com/tianocore/edk2/blob/edk2-stable202502/OvmfPkg/RUNTIME_CONFIG.md) en [NetworkComponents](https://github.com/tianocore/edk2/blob/edk2-stable202502/OvmfPkg/Include/Dsc/NetworkComponents.dsc.inc).
