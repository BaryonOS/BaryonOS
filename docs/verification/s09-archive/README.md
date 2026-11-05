# S09 — hostbewijs herstelkopie-/exportarchief (5 november 2026)

**Basis:** main `9bab3d7aeaac5983b46234f53d037df1acd15af8`, branch
`storage/s09-backup-archive`. **Bewijsniveau:** hostmodel (echte volume-client
en -service met gesimuleerde schijf). Geen native QEMU, geen gebruikerstaak, geen
fysieke hardware. S09 blijft **open**; dit bewijs kent geen acceptatiepunten toe.
Specificatie: [S09-BACKUP-ARCHIVE.md](../../implementation/S09-BACKUP-ARCHIVE.md).

## Bestanden (sha256)

| Bestand | sha256 |
|---|---|
| services/volume/src/archive.rs | `df9988210e12c95bc9a45e6cca0babb69f9fc242ca2e620dc1f5f6bdf287201b` |
| services/volume/tests/archive.rs | `80fa05143e7bf56cef789ebb0f0ca84d4319fcb7bbd17d03d47e12c04eec7479` |
| services/volume/src/lib.rs | `8381d94c77080519751f30bedd398348c32448d5645b89a3ddc3261b9bf2226a` |
| docs/implementation/S09-BACKUP-ARCHIVE.md | `08452612f947b1cacf3a196d6a699c78c30d2692a691eecf0d9d0b54bd4c72a5` |

## Uitgevoerde commando's (Rust 1.90.0)

Na de CodeRabbit-review opnieuw gedraaid op de gnu-hosttarget, met de lokale
toolchain uit `.tools/nieuw-os` (eigen `cc`-linker). CI draait dezelfde
gnu-route via `.github/workflows/ci.yml`, die `-p volume-core` al dekt.

| Commando | Exit | Resultaat |
|---|---|---|
| `cargo test --locked -p volume-core --test archive` | 0 | 17/17 |
| `cargo test --locked -p volume-core` | 0 | 66 geslaagd, 0 gefaald (alle targets), waarvan 17 nieuw |
| `cargo clippy --locked -p volume-core --all-targets -- -D warnings` | 0 | geen waarschuwingen |
| `cargo build --locked -p volume-core --target x86_64-unknown-none` | 0 | freestanding no_std build |
| `cargo fmt -p volume-core -- --check` | 0 | — |

## Mutatieproeven

Opnieuw uitgevoerd na de review-fixes (de oorspronkelijke telling van 11 bij een
lijst van 10 was niet meer te reconstrueren). Elke mutatie is tijdelijk in
`archive.rs` aangebracht, daarna is `cargo test --locked -p volume-core --test archive`
gedraaid en is het bestand bytegelijk teruggezet (sha256 hierboven gecontroleerd).

**Laten elk ten minste één test falen (11 van 13):**
- naamconflict bij restore genegeerd → `restore_never_overwrites_objects_names_or_collections`;
- EOF-controle na de trailer weg → `every_truncation_flip_and_trailing_byte_is_rejected`;
- `Protocol` → `Corrupt` bij een te hoge telling op de EOF-probe → `overlong_count_at_end_of_stream_is_a_protocol_error`;
- `Protocol` → `Corrupt` bij een te hoge bron-telling in `In::exact` → `overlong_source_count_is_a_protocol_error`;
- eindcontrole op de generatie (`list`) weg → `concurrent_change_makes_export_stale`;
- inhoudssom van de dienst bij export weg → `export_rejects_content_that_contradicts_the_service_sum`;
- vreemde doelcollectie als bruikbaar behandeld → `restore_never_overwrites_objects_names_or_collections`;
- inhoudssomvergelijking bij `Unknown` weg → `unknown_commit_with_another_content_sum_is_not_claimed`;
- `bare`-markering weg → `failed_metadata_is_reported_as_bare`;
- weigering van revisie 0 weg → `resealed_non_canonical_records_are_rejected`;
- opvulling van objectnamen niet gecontroleerd → `resealed_non_canonical_records_are_rejected`.

**Overleven (2):**
- de revisievergelijking bij `Unknown` helemaal weg (de controles op naam,
  lengte en inhoudssom blijven staan). Een revisie die één te ver ligt
  (`checked_add(2)`) wordt wel gevangen door `unknown_commit_is_resolved_by_stat`;
- de controle `stat.length != list.length` bij export.

Beide zijn verdediging tegen een andere schrijver of een afwijkende dienst en
worden in de huidige tests niet los geraakt.

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
