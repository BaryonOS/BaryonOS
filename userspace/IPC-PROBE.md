# Eigen IPC-userfixture U04

`ipc_probe.rs` is een afzonderlijk synthetisch ring-3-programma voor M4. Het vervangt de M3-probe niet. De fixture voert werkelijke `int 0x80`-syscalls uit; alleen de kernelobserver beslist over KS-09–16. Deze tekst beschrijft de afgesproken fixture-interface en geplande echte cases, geen reeds geslaagde QEMU-proeven.

## Bouwen en controle

Root beheert de bestaande buildtool:

```powershell
python tools/build_users.py --kind ipc
python -B -X utf8 userspace/tests/test_ipc_probe.py --elf build/users/ipc_probe.elf
```

Input: `userspace/ipc_probe.rs`, `userspace/ipc_linker.ld`. Output: `build/users/ipc_probe.elf`, `build/users/manifest-ipc.json`. Embedvariabele: `NOS_IPC_ELF`. Directe Rust1.90.0-flags zijn gelijk aan M3: x86_64-unknown-none, no_std, panic=abort, opt-level=2, code-model=small, relocation-model=static, no-redzone=yes, rust-lld, eigen absolute `-T`-linkerpad, --no-pie, --build-id=none, max-page-size=0x1000. Geen dependencies, allocator, hostlibc, netwerk, hostcalls of port-I/O.

## Geheugeninterface

| Virtueel bereik | ELF-rechten | Inhoud |
|---|---|---|
| `0x40000000` | RX | `_start` en interpreter |
| `0x40010000–0x40011000` | RW | Controlwoorden, rows en resultaten |
| `0x40011000–0x40012000` | RW | `msg`, 4096 bytes |
| `0x40012000–0x40013000` | RW | `out`, 4096 bytes |
| `0x40013000–0x40014000` | R | Nulgevulde RO-testpagina, NX te mappen door kernel |

Alle bereiken zijn halfopen. Het ELF heeft drie PT_LOAD-segmenten: RX, 12288 bytes RW, 4096 bytes R. ELF-flags bewijzen geen daadwerkelijk ingestelde pagingrechten. Ieder proces krijgt eigen fysieke frames; barriers staan eveneens in afzonderlijke privéframes.

Controlwoorden zijn little-endian u64 op `0x40010000`:

| Offset | Inhoud |
|---|---|
| 0 | Mode: 0 worker; 1 copy; 2 wire; 3 snapshot; 4 handles; 5 rights; 6 revoke; 7 quotas; 8 lifecycle |
| 8 | Privécounter; verhoogd bij iedere syscall en expliciete crashop |
| 16 | Privépatroon; initiële kopie blijft in r13 en wordt tussendoor gecontroleerd |
| 24 | Target/scratch, door manifest bepaald |
| 32 | Role: A=0, B=1, C=2 |
| 40 | Aantal volledig afgeronde rows |
| 48 | Status: 0 actief, 1 rows afgerond, `0x100+rowindex` fixturefout |
| 56 | Main: eigen endpointroot, rechten31 |
| 64 | Aux: initiële lokale capability naar volgende rol, rechten29 (geen RECV) |
| 72 | Foreign: numerieke peerhandle; geeft geen toegang tot peertabel |
| 80 | Laatste RAX/diagnose |
| 88 | Rowaantal, maximaal32 |
| 96 | Eigen barrierwoord; kernelobserver wijzigt dit pas na geobserveerde echte gebeurtenissen |
| 104 | Gereserveerd, niet gebruikt door interpreter |
| 112 | Pogingenteller, maximaal32768 |

De interpreter voert vooraf `abi_info(0)` uit en controleert RAX=0/RDX=`1<<32`. Na status1 blijft het proces counter/yield-worker; andere processen kunnen daardoor werkelijk na afronding, exit of fault van een peer voortgang tonen. Bij een fixturefout bewaart de interpreter diagnose/status en doet gewone `exit(0xe4)`. Geen usercode schrijft kernelmarkers of bepaalt het kerneltestresultaat.

## Rows en werkelijke resultaten

32 rows beginnen op `0x40010100`; iedere row bevat tien u64's, exact80 bytes:

```text
op @0
a0,a1,a2,a3,a4,a5 @8,16,24,32,40,48
expected_RAX @56
expected_RDX @64
flags @72
```

Negatieve verwachte foutcodes zijn u64-tweecomplementwaarden. Resultslot i staat op `0x40010c00 + 16*i`: werkelijke RAX op+0 en werkelijke RDX op+8. Slots worden door usercode gevuld uit de echte syscallregisters of een gewone `load64` uit eigen ontvangen data. De kernel fabriceert geen syscallresultaten.

| Flags | Betekenis |
|---|---|
| Bits0–5 | Het betreffende argument is een resultindex; laad de werkelijke RDX van dat slot |
| Bit6 | Vergelijk de dynamische RDX niet met een vaste waarde; bewaar haar wel en controleer RAX |
| Bit7 | `expected_RDX` is zelf een resultindex |
| Bit8 | Alleen bij EAGAIN: echte yield en dezelfde row opnieuw uitvoeren |
| Bits32–47 | Herhalingen van dezelfde row, 0 betekent1; maximaal1024 |
| Overig | Ongeldig; geaccepteerd masker `0x0000ffff000001ff` |

Iedere herhaling resolveert de argumenten opnieuw en overschrijft uitsluitend de eigen resultslot met de laatste echte uitkomst. Een verwijzing naar de eigen resultslot kan een derivechain voortzetten, nadat een gewone user-write die slot met een echt eerder ontvangen/afgeleid handle heeft geïnitialiseerd. Rowprogress stijgt pas na alle succesvolle herhalingen. EAGAIN- en barrierretries tellen mee in de begrensde pogingenteller.

Voorbeeld van echte handlevervolgactie: `ipc_recv(main,out,320)` → `load64(out+32)` met flag bit6 → schrijf die geladen RDX via bit1 naar `msg+32` → `ipc_send(aux,msg,320)`. Sluit afgeleide handles eveneens via hun werkelijke RDX-resultslot, niet via veronderstelde slot/generationgetallen.

## Interpreteroperaties

`op < 0x100` is rechtstreeks het syscallnummer. Argumenten gaan naar RDI/RSI/RDX/R10/R8/R9; RAX/RDX worden daarna vastgelegd. Dit omvat gewone syscalls0/1/2, cap_close10h/cap_derive11h/cap_revoke12h, ipc_send20h/ipc_recv21h en echte onbekende nummers voor ENOSYS-proeven.

| Fixtureop | Argumenten en effect |
|---|---|
| `0x100` WRITE64 | a0=adres, a1=waarde; gewone private 8-byte store |
| `0x101` FILL | a0=adres, a1=byte0..255, a2=lengte0..4096 |
| `0x102` ASSERT64 | a0=adres, a1=verwachte waarde; gewone private read/vergelijking |
| `0x103` LOAD64 | a0=adres; resultaat RDX uit eigen geheugen, RAX=0 |
| `0x104` WAIT64 | a0=eigen adres, a1=verwachte waarde; anders echte yield en retry |
| `0x105` UD2 | Expliciete receiverfault voor lifecycleproef; geen normale terugkeer |

Writes/FILL mogen uitsluitend resultslots `0x40010c00–0x40010e00` of RW-buffers `0x40011000–0x40013000` raken. Zij kunnen geen rows, status, barrier of mode veranderen. Readops zijn beperkt tot `0x40010000–0x40014000`. Alle lokale optellingen controleren carry; een verkeerd fixturescript resulteert in een fixturefout. Een willekeurige syscallpointer blijft een integer, zodat juist de kernel haar echte bereik/rechten moet controleren. Misalignment is voor deze byte-/x86-geheugenoperaties toegestaan.

## Message320

Het OS-wirecontract blijft ongewijzigd: sender_pid u64@0 (send0), tag u64@8, len u32@16, cap_count u32@20, reserved u64@24 (0), twee `{handle:u64,rights:u64}`-paren@32/48, data[256]@64. Totaal320 bytes. Een WRITE64 op+16 combineert daarom `len | (cap_count<<32)`. Niet-gebruikte descriptors/payloadbytes blijven nul. De RO-pagina bevat aanvankelijk een geldige lege canonical message; ontvangst daar moet wegens schrijfrechten mislukken.

Bevestigde codes: EMSGSIZE(-11) voor size≠320/len257/capcount3; EINVAL(-1) voor sender/reserved/ongebruikte descriptor/tail of onbekende rightsbit32; EACCESS(-5) voor noSEND/noDERIVE/rechtenvergroting/transfer zonderTRANSFER/transferRECV. Revoked(-10) gaat vóór Closed(-9). Header- en capabilityvalidatie vóór userbuffer is onderdeel van de kernelobserverproef.

## Afgesproken case-map voor kernelmanifest en observer

| Mode / KS | Echte handelingen | Door kernel te bewijzen |
|---|---|---|
| 1 / KS09 | Send/recv met null, supervisor, noncanonical, overflow, stack/heap-eindcross; out→RO-cross op `0x40012f80`; misaligned geldige buffers | EFAULT of eerdere EMSGSIZE; volledige output/queue/captable vóór/na gelijk bij mislukking; na geldige recv normale aflevering |
| 2 / KS10 | Sizes0/319/321/MAX; len257/capcount3; niet-nul sender/reserved/unusedcap/tail; buffers+1 | Exacte foutvolgorde/codes, geen mutatie; geldige misalignment blijft toegestaan |
| 3 / KS11 | Send bekende bytes; hook verandert originele len/payload pas na snapshot; recv; onbekende thread/shared/unmap-nummers | Alleen originele snapshot wordt gebruikt; ontbrekende API's ENOSYS; geen claim over echte SMP-races |
| 4 / KS12 | Numerieke peerhandle, slot0/ongeldige gen; derive→close→stale; bewuste lokale collision | Lookup uitsluitend eigen tabel; EBADHANDLE waar ongeldig; collision verleent hoogstens bestaande eigen autoriteit |
| 5 / KS13 | Minder/zero rights derive; extra32/vergroting/noDERIVE/noSEND; transfer zonder8/metRECV | EINVAL/EACCESS volgens contract, geen nieuwe node/slot/queue bij fout |
| 6 / KS14 | A maincap13→B via aux; queued revoke vóórBrecv; aparte delivered A→B→C met echte ontvangen handles; A revoke | Queued bericht volledig geannuleerd; afgeleverde B/C-handles EREVOKED; eerdere payload blijft leesbaar |
| 7 / KS15 | Acht depthderives plus negende; close echte outputs; 253 derives bij init2slots; laatste close/rederive;16send/17e/drain/send | Depth/capslots/queuequota en capaciteitsterugkeer via echte calls; geen gedeeltelijke transacties |
| 8 / KS16 | Drie rondes: ownerclose; senderexit na queued delegation; receiverUD2 na queued delegation | LaatsteRECV sluit endpoint; issuer-exit revoke/cancel; refs/resources worden opgeruimd, andere processen lopen door |

Bij lifecycle receiver-exit kan A.aux EREVOKED opleveren als die bootstrapnode door B is uitgegeven; dat heeft voorrang op ECLOSED. Een gewone owner-close zonder exit geeft de peers ECLOSED. Kernelmanifest en Engine-metadata bepalen het concrete verwachte geval; de userfixture verandert deze precedence niet.

De kernelobserver controleert daadwerkelijke syscallnummers, argumenten en resultaten, volledige relevante outputbytes, FIFO, captables/nodes en resources. Status1 is alleen userfeedback. De globale1024-nodegrens en generationexhaustie worden aanvullend in het hostengine-model getest; zij tellen zonder uitgevoerde extra QEMU-case niet als fysiek afgedwongen gastbewijs.

## Uitgevoerd en herstel

De rootbouw en afzonderlijke hostartifacttests staan in `verification/U04-HOST-2026-10-27.md` en `verification/ipc-host-evidence.json`. Opcode- en ELF-controle bewijzen geen uitgevoerde syscall. De echte EXCHANGE/caseboots en M2/M3-regressies worden door root/kernelagent uitgevoerd en afzonderlijk vastgelegd.

Geen M3-probe, M3-test, roottool of kernelbestand is vanuit U04 gewijzigd. Herstel: neem uitsluitend deze aparte IPC-fixturebestanden en de door root beheerde optionele NOS_IPC_ELF-buildkoppeling terug; de M3-fixture blijft zelfstandig bruikbaar.
