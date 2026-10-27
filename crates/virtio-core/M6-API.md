# C06 — begrensde WRITE/FLUSH-core

Bouwbasis448974b, contract docs/implementation/M6A-WRITE-RECOVERY.md v1.
Safe Rust/no_std, nul dependencies, geen heap/unsafe/MMIO of userpointers.
Dit document beschrijft de pure interface; hardwareordering, echte resetack,
eigenaarschap/pinning en usercopy blijven verantwoordelijkheid van de kernel.

## Getypeerde capabilities

Endpoint ALL_RIGHTS blijft31; Block READ1, WRITE32, FLUSH64, DERIVE4,
TRANSFER8, REVOKE16. Block-bit2 blijft ongeldig. KNOWN_RIGHTS bevat127,
maar typespecifieke validatie voorkomt Endpoint-WRITE/FLUSH of Block-RECV.

`create_block_device(owner,scope)` blijft root29 maken.
`create_block_device_with_rights(owner:u64,scope:BlockScope,rights:u64)` maakt
uitsluitend expliciet gekozen rechten binnen125; manifest kan root125 maken
en de dienst97 delegeren. `check_block` controleert type vóór recht en geeft
dezelfde immutable scope/opaque objectkey. FLUSH heeft een devicebreed
cache-effect; WRITE/READ moeten door de kernel op sectorscope worden getoetst.
Geen nieuwe user-creatiesyscall, objectquota of lifetime-infrastructuur.

## Operaties en vaste descriptorchains

```rust
enum Operation {
    Read { sector:u64, offset:u64, length:u64 },
    Write { sector:u64 },
    Flush,
}
struct DescriptorChain { pub entries:[Descriptor;3], pub count:u8 }
fn negotiate_writable_features(offered:u64) -> Result<u64,Error>;
fn descriptor_plan_for(op:Operation,pool:DmaPool,ram_end:u64)
    -> Result<DescriptorChain,Error>;
fn validate_descriptors_for(chain:&DescriptorChain,op:Operation,
    pool:DmaPool,ram_end:u64) -> Result<(),Error>;
fn header_for(op:Operation) -> [u8;16];
fn validate_used_for(op:Operation,head:u16,old:u16,new:u16,
    id:u32,len:u32,status:u8) -> Result<Completion,Error>;
```

WRITE is altijd512B/offset0; FLUSH is header(type4,sector0)→status1 en heeft
geen datadescriptor. READ heeft header→512device-WRITE→status, WRITE heeft
header(type1)→512device-READ→status. Chaincount3/3/2; ongebruikte descriptor
bij FLUSH is exactnul en wordt meegevalideerd. Success used.len513/1/1;
IO/UNSUPP geeft geen payload. De decoder produceert nooit Unknown.

Het writable profiel vereist VERSION1 en FLUSH9, weigert RO/ACCESS_PLATFORM,
en onderhandelt alleen VERSION1|FLUSH. CONFIG_WCE wordt niet onderhandeld.
Bestaande M5 feature-/descriptor-/header-/usedwrappers blijven readonly.

## Publicatie en onzekere afloop

```rust
Policy::prepare(binding:Binding,op:Operation,capacity:u64,now:u64,timeout:u64)
    -> Result<Request,Error>;
Policy::publish(epoch:u64,ticket:u64) -> Result<(),Error>;
Policy::reject_prepared(binding:Binding,ticket:u64) -> Result<(),Error>;
Policy::exit(pid:u64) -> Option<Request>;
```

Request behoudt oude velden en voegt `operation:Operation` en `published:bool`
toe. Nieuwe route: Ready→Prepared→Published. Legacy submit(Read) blijft
Phase::Pending en is conservatief al exposed. Nieuwe transports roepen publish
direct vóór avail.idx-publicatie aan; niet pas bij notify. Prepared-reject
publiceert niets en verwijdert het ticket. Geen onbetrouwbare velden ná snapshot
of publicatie opnieuw lezen; deze crate kent de daadwerkelijke userbuffer niet.

PID/objectkey/epoch/ticket zijn monotone bindingen; Request.operation bepaalt
het vereiste pollrecht. De kernel herautoriseert dat recht op een actuele cap.
Een andere geldige cap naar hetzelfde object binnen dezelfde PID blijft bruikbaar.

Published WRITE/FLUSH met IO/UNSUPP wordt Completion::Unknown. Timeout of
start_reset zonder geobserveerde completion bewaart eveneens Unknown. Een
gevalideerde succesvolle WRITE blijft Success, maar betekent geen durability;
daarvoor is een latere geslaagde echte FLUSH nodig. Geen automatische replay.
Voltooide resultaten verlopen niet door een wachtende gebruikerspoll.

Resetack verhoogt transportepoch maar behoudt ongeconsumeerde mutatorresultaten
met oorspronkelijk ticket/epoch. Peek/consume accepteert uitsluitend hun exacte
binding; nieuwe prepare blijft Busy totdat het resultaat is verwerkt. Unknown
kan tijdens Resetting/Quarantined worden opgehaald, maar consume heft die phase
niet op. Exit verwijdert uitsluitend het resultaat van de stervende PID en houdt
noodzakelijke reset/quarantaine. Prepared-exit heeft geen device-effect. Epoch-
of ticketwrap is verboden; terminal Unknown blijft leesbaar bij epochuitputting.

## Hostbewijs en grenzen

30 capability- en21 virtio-hosttests behouden alle38 oorspronkelijke tests en
voegen13 transactietests toe. Eén oorspronkelijke unknown-bit-probe is vanwege
de vooraf gekozen WRITE32-toewijzing expliciet bijgewerkt: legacyroot29+WRITE32
geeft nu EACCESS; bit128 geeft nog EINVAL. Beide vergelijken de volledige Engine
vóór/na de fout. De test en generationretirementchecks zijn behouden.

Nieuwe tests dekken getypeerde WRITE/FLUSH-grants, alle256 kleine rightsmaskers,
revoke/IPC/exit/generation/quota, alle descriptorvelden inclusief ongebruikte
FLUSH-entry, operationele usedlengtes, Prepared/Published, Unknown over reset,
consume tijdens reset/quarantaine, eigenaar-exit en no-wrap.

Commando's/exits/logs/hashes staan lokaal in verification/m6-report.json;
root bewaart de compacte versie in docs/verification. Deze hosttests bewijzen
geen echte input512-snapshot, PCI/DMA/FLUSH, QEMU-kill/recovery of powerloss.
VIRTIO gebruikte entries hebben geen epochnonce; softwarebinding detecteert
geen kwaadwillige device-replay. Kerneltransport en QEMU/backend blijven TCB.

Herstel vereist een consistente terugkeer naar448974b van beide crates en
hun nieuwe kernelcallers; geen los terugdraaien van één gedeelde interface.
Er zijn geen externe dependencies of lockfilewijzigingen toegevoegd.
