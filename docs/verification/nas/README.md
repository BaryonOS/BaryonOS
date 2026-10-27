# N01 — vastgelegd verificatiebewijs

27 oktober 2026. Ontwikkelbasis `a96ad45adb044177e18670cd38624ea2a671db3d`, featurebranch `codex/nas-private-sharing`.

- `report.json`: werkelijke opdrachten, uitkomsten, runtime, bronhashes en expliciete grenzen.
- `tests.txt`: gecombineerde suite, **89/89** (50 service, 39 browserstate), exit0.
- `demo.json`: eindige echte encryptie-/ciphertext-/adminverwijderproef, exit0.
- `review.md`: concrete bevindingen en reparaties.

Dit bewijst een lokaal hostprototype en browserlogica. Het bewijst geen native netwerkverbinding, TLS, echte NAS-installatie of productiegeschiktheid. De nieuwe grafische browsercontrole is niet uitgevoerd; de Browser-plugin kon zonder oorspronkelijke browsercontext niet starten. Er zijn geen echte gebruikersbestanden, NAS-instellingen of routers gewijzigd.

N01 voegt automatische beleidskeuze voor LAN→internet→LAN toe met één bestaande, peergebonden toestemming. De planner verleent geen rechten en de UI bewaakt afzonderlijke lokale beheeractivatie. Beide blijven prototypes; persistente clientkeys, rotatie, multi-device en native integratie zijn productvereisten.

De onafhankelijke AI-reviewherhalingen overlappen de 89 tests en worden niet als extra unieke tests opgeteld. Bestaand M1–M6a-bewijs blijft afzonderlijk.
