# L02 — native quantum-jobinterface

Dit profiel voegt een echte gebruikersprocesroute toe boven de bestaande
begrensde qsim-port. De kernel voert geen simulatorlogica uit: hij maakt twee
processen, twee endpointobjecten en uitsluitend de minimale SEND/RECV-grants.

De **caller** verstuurt canonieke 256-byte `QJ01`-requests. De **dienst**
valideert afzender, tag, lengte en requestcodec, voert de bestaande
`qsim-native` adapter uit en stuurt een begrensd reply terug. Tijdens een job
controleert de `Control`-callback dezelfde requestendpoint op een Cancel. Het
aangevraagde `work_budget` wordt bij ieder checkpoint afgedwongen.

De acceptatie voert vier jobpaden uit: Bell met 4.096 shots; een zes-qubit
X-circuit; een Cancel die via IPC tijdens uitvoering wordt opgehaald; en een
job die door zijn werkbudget als deadline wordt beëindigd. Resultaten worden
in vaste pagina's van maximaal zestien histogramwaarden teruggegeven.

De dienst gebruikt een private 64-KiB-arena. Na iedere beëindigde `execute`
zijn alle tijdelijke Vecs vernietigd; pas daarna wordt de monotone arena
teruggezet voor de volgende job. De latere kernelpoort moet aantonen dat beide
processen, capabilities/endpoints, queues en frames volledig worden opgeruimd.

Dit is een lokale software-simulator/jobinterface op het eigen OS. Het is geen
fysieke QPU, geen quantumversnelling en geen cloudroute.
