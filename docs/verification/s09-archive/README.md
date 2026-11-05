# S09 — hostbewijs herstelkopie-/exportarchief (5 november 2026)

**Basis:** main `9bab3d7aeaac5983b46234f53d037df1acd15af8`, branch
`storage/s09-backup-archive`. **Bewijsniveau:** hostmodel (echte volume-client
en -service met gesimuleerde schijf). Geen native QEMU, geen gebruikerstaak, geen
fysieke hardware. S09 blijft **open**; dit bewijs kent geen acceptatiepunten toe.
Specificatie: [S09-BACKUP-ARCHIVE.md](../../implementation/S09-BACKUP-ARCHIVE.md).

## Bestanden (sha256)

| Bestand | sha256 |
|---|---|
| services/volume/src/archive.rs | `08561b0c8e81430aeacfd56245fc0fef9016a5a534f03da3c5686ed8eb1addfd` |
| services/volume/tests/archive.rs | `5b80f59d3c2323ba9f40e8dc86d46ae059304498ef062d203ee5941a34a10cce` |
| services/volume/src/lib.rs | `8381d94c77080519751f30bedd398348c32448d5645b89a3ddc3261b9bf2226a` |
| docs/implementation/S09-BACKUP-ARCHIVE.md | `4484b3da2ba218fdf6bc5dceb7cc97d66999c1d660aa45c7611501f8d071f589` |

## Uitgevoerde commando's (Rust 1.90.0)

De lokale WSL-omgeving heeft geen `cc`. De host-tests draaiden daarom op
`x86_64-unknown-linux-musl` met `rust-lld`
(`-C linker-flavor=ld.lld -C link-self-contained=yes`). CI draait de gewone
gnu-route via `.github/workflows/ci.yml`, die `-p volume-core` al dekt.

| Commando | Exit | Resultaat |
|---|---|---|
| `cargo +1.90.0 test --locked --target x86_64-unknown-linux-musl -p volume-core --test archive` | 0 | 15/15 |
| `cargo +1.90.0 test --locked --target x86_64-unknown-linux-musl -p volume-core` | 0 | 64 geslaagd, 0 gefaald (alle targets) |
| `cargo +1.90.0 clippy --locked --target x86_64-unknown-linux-musl -p volume-core --all-targets -- -D warnings` | 0 | geen waarschuwingen |
| `cargo +1.90.0 build --locked -p volume-core --target x86_64-unknown-none` | 0 | freestanding no_std build |
| `cargo +1.90.0 fmt -p volume-core -- --check` | 0 | — |

## Mutatieproeven

Elke mutatie is tijdelijk aangebracht en daarna bytegelijk teruggezet.

**Laten elk ten minste één test falen (11):**
- naamconflict bij restore;
- EOF na de trailer;
- de eindcontrole op de generatie (`list`);
- de controle op de inhoudssom van de dienst;
- de mapping naar een vreemde collectie;
- `Protocol` bij een te hoge bron-telling;
- de revisiecontrole bij `Unknown`;
- de `bare`-markering;
- de weigering van revisie 0;
- de opvulling van namen.

**Overleven (2):** de vergelijking van de inhoudssom bij `Unknown` en de controle
`stat.length != list.length` bij export. Beide zijn verdediging tegen een
andere schrijver of een afwijkende dienst en worden in de huidige tests niet
geraakt.

## Review

Er is één onafhankelijke AI-review gedaan (geen menselijke audit), zonder blokkerende bevindingen. Verwerkt:
- rapport bij een mislukte write of META (`bare`, `uncollected` pas na de commit);
- `Unknown` opgelost via `stat`;
- dubbele namen toegestaan, net als in de volume, zodat een export altijd zijn eigen
  verify doorstaat;
- de lengtecontrole bij export;
- tests met opnieuw verzegelde, niet-canonieke records.

Als contract gedocumenteerd: één schrijver op het doeldomein tijdens de restore,
en het gedrag bij een tombstone-slot.

## Open voor S09

- Een native back-upportaal met een door de gebruiker gekozen doelvolume en de bijbehorende capability.
- Een native QEMU-proef op een tweede virtuele schijf.
- Export naar gangbare bestandsformaten.
- Een versleutelde variant voor NAS of provider (B02/P01).
- Een gebruikerstaak.
