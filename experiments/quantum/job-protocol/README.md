# Native quantum-jobrequests — voorstel gebruikersdienst-v1

Dit zelfstandige pakket implementeert het centraal afgestemde interne
requestcontract `QJ01`, versie 1. Het kan één bestaande IPC-payload van exact
256 bytes getypeerd coderen en strikt valideren, zonder allocator, `unsafe`
of externe dependency. Het is een eerste bouwsteen voor jobs buiten de
vaste native selftest.

**Uitvoering ontbreekt nog.** Er is geen jobdispatch, wachtrij, dienst,
replyprotocol of werkende cancel-/status-/resultaatoperatie toegevoegd.
De codec controleert het aangevraagde werkbudget; zij dwingt tijdens het
rekenen geen quota af. De kernel-IPC, syscalls, capabilities, opslagformaten,
de bestaande qsim-core en de native selftest veranderen niet. Het bestaande
host-JSON-protocol `qsim-v0` blijft een afzonderlijke adapter.

## Exact payloadcontract

Alle meervoudige integers zijn little endian. Niet gebruikte en gereserveerde
bytes moeten nul zijn; de decoder negeert geen extra velden of trailing data.

| Bytes | Veld | Regel |
|---|---|---|
| 0–3 | magic | Exact `QJ01` |
| 4–5 | version, u16 | Exact 1 |
| 6 | operation, u8 | 1 Submit, 2 Cancel, 3 Status, 4 ResultPage |
| 7 | qubits, u8 | Submit: 1–6; anders nul |
| 8–15 | job_id, u64 | Altijd niet-nul |
| 16–23 | request_id, u64 | Altijd niet-nul |
| 24–31 | seed, u64 | Alleen Submit; iedere u64 toegestaan |
| 32–35 | shots, u32 | Submit: 1–4.096; anders nul |
| 36–37 | gate_count, u16 | Submit: 0–64; anders nul |
| 38–39 | reserved | Altijd nul |
| 40–47 | work_budget, u64 | Submit: 1–1.048.576; anders nul |
| 48 | result_page, u8 | Alleen ResultPage: 0–3; anders nul |
| 49–63 | reserved | Altijd nul |
| 64–255 | maximaal 64 gates | Drie bytes per gate; alle ongebruikte bytes nul |

Een gate is `[operation, target, control]`. De gatecodes zijn 1 H, 2 X,
3 Z, 4 S en 5 CX. Alleen CX gebruikt `control`; voor de andere gates moet
die byte nul zijn. Iedere target en iedere CX-control moet kleiner zijn
dan `qubits`, en CX-control moet verschillen van target. Validatie vindt
plaats op de u8-indices voordat een toekomstige backend ze verbreedt.

Cancel en Status bevatten naast de gemeenschappelijke header alleen de
twee ID's. ResultPage bevat bovendien de page-index. Ook voor deze
operaties moeten alle overige bytes nul blijven.

Er staat **geen caller-PID of autoriteit in de payload**. Beide ID's zijn
correlatievelden; kennis van een job-ID geeft geen jobrechten. Een toekomstige
userspace-dienst moet de actor aan het geauthenticeerde IPC-envelope binden,
rechten controleren en request-ID's volgens haar afgesproken lifecycle
verwerken. Deze codec alleen bewijst die koppeling niet. De seed blijft
uitsluitend invoer voor herhaalbare simulatie, geen sleutel of entropybron.

## Rust-API

`Identifiers::new` vereist twee niet-nul u64's. `Submit::new` valideert de
limieten en gates vóór het kopiëren naar een vaste array. `ResultPage::new`
controleert de page-index. Hun velden zijn privé zodat veilige aanroepers
geen ongevalideerde request kunnen encoderen.

```rust
use qsim_job_protocol::{decode, encode, Gate, Identifiers, Request, Submit};

fn encode_example() -> Result<[u8; 256], qsim_job_protocol::Error> {
    let ids = Identifiers::new(1, 1)?;
    let request = Request::Submit(Submit::new(
        ids, 2, 0, 4096, 100_000,
        &[Gate::H { target: 0 }, Gate::Cx { target: 1, control: 0 }],
    )?);
    let payload: [u8; 256] = encode(&request);
    assert_eq!(decode(&payload)?, request);
    Ok(payload)
}
```

`decode(&[u8])` accepteert uitsluitend 256 bytes. `encode` schrijft altijd
een canoniek `[u8; 256]`. Lokale `Error`-varianten beschrijven afwijzing; zij
leggen geen antwoord-, syscall- of on-disk-foutcodering vast.

De Rust-typen zijn geen wirestructs: hun geheugenindeling wordt nergens
gekopieerd of als ABI gebruikt. `Request` bevat opzettelijk de volledige
vaste gatebuffer; daarom heeft uitsluitend dit enum een gemotiveerde lokale
Clippy-uitzondering voor variantgrootte. Een `Box` zou juist de verboden
allocatorafhankelijkheid toevoegen.

## Controles en reproduceren

```bash
cargo +1.90.0 test --manifest-path experiments/quantum/job-protocol/Cargo.toml --locked --offline
cargo +1.90.0 clippy --manifest-path experiments/quantum/job-protocol/Cargo.toml --all-targets --locked --offline -- -D warnings
cargo +1.90.0 build --manifest-path experiments/quantum/job-protocol/Cargo.toml --lib --release --target x86_64-unknown-none --locked --offline
```

Gebruik een eigen `CARGO_TARGET_DIR` bij parallel bouwen. De workspace staat
los van de kernel en van de bestaande quantum-workspaces.

De 13 hosttests controleren exacte golden bytes, alle vier operaties en
vijf gatecodes, maximale arrays, minima/maxima, alle 256 afgekorte lengtes,
te lange invoer, header- en ID-fouten, iedere gereserveerde of ongebruikte
bytepositie, bounds en CX-regels. Een aanvullende eigenschap binnen die
tests controleert 6.400 enkele-bytemutaties: iedere geaccepteerde mutatie
moet zonder normalisatie identiek terugencoderen.

De library compileert native voor `x86_64-unknown-none`; dat is
**compilatiebewijs, geen uitgevoerde OS-job of VM-run**. De exacte uitgevoerde
commando's, exitstatussen, bron- en artifacthashes staan onder
`docs/verification/parallel-i08`.

## Kleinste vervolgstap

Verbind deze codec met een afzonderlijk geautoriseerde userspace-dispatcher,
een afgesproken replycontract en de bestaande qsim-core. Die integratie moet
actorbinding, lifecycle, werkelijk quota-/cancelgedrag en resultaatpagina's
native testen. Voeg de nieuwe brondependency centraal aan de OS-bronbinding
toe zodra de OS-build haar daadwerkelijk gebruikt. De codec wordt nog niet
door de bestaande OS-image ingelinkt. L02 blijft daarom open.
