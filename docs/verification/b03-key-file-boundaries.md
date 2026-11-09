# B03 — grenzen van gegenereerde sleutelbestanden

Basis: `8331b69b`, branch `coderabbit/issue-41-key-file-boundaries`.
Oorspronkelijk issue: https://github.com/simonvanieperen/BaryonOS/issues/41.

`update-tool keygen` maakte seeds met de standaard bestandsrechten aan.
Bij umask `000` leverde dit modus `0666` op. Een gebroken symlink naar
het eerste seedbestand werd gevolgd. Beide nieuwe regressietests faalden
op de ongewijzigde implementatie (Cargo exit 101).

De correctie gebruikt `create_new(true)` voor alle vijf bestanden en
Unix-modus `0600` bij aanmaak van beide seeds. Niet-Unix private aanmaak
wordt expliciet geweigerd; de bestaande entropyroute vereist reeds
`/dev/urandom`. De tests gebruiken uitsluitend tijdelijke TEST-sleutels.

Uitgevoerd op de Linux-host met Rust 1.90.0, alle onderstaande controles exit 0:

```sh
cargo +1.90.0 test --locked -p update-core -p update-tool --features update-core/test-signing-anchors
cargo +1.90.0 clippy --locked -p update-core -p update-tool --all-targets --features update-core/test-signing-anchors -- -D warnings
python3 -B -m unittest discover -s tests -p 'test_update_*.py'
python3 tools/progress.py --check
```

Resultaat: 18 Rust-tests en 14 Python-tests slagen. Dekking omvat permissies
bij umask `000`, levende en gebroken symlinks op alle vijf uitvoerpaden,
handtekening-/verificatie-roundtrip en bestaand sleutelbeleid.

Bewijsgrens: hosttoolcorrectie, geen native of productieacceptatie.
QEMU-gate `tools/test_update.py`, fysieke hardware en productie-rotatie
zijn niet uitgevoerd. Bovenliggende mappen blijven vertrouwde invoer;
een schrijfprobleem kan nog een gedeeltelijke sleutelset achterlaten.
Er is geen nieuwe afhankelijkheid, cryptoprimitief of productiekey.
Rubric blijft 41/100. Issue #41 blijft open.
