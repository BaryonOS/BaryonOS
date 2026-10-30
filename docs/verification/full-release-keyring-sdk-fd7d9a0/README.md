# Native KEY1 SDK-client — begrensd integratiebewijs

Geteste schone broncommit: `fd7d9a04e34ed6124535512c1627300ce7ee546c`.
Deze omvat de gereviewde client (developmentcherry `94adbe4`, inhoud van
`3f07b1c`), appwijziging `e5f9250` en expliciete SDK-aansluiting `fd7d9a0`.
Bronfingerprint v3:
`b417246fff20a716f0a6f711d5b95613cc7bc2fc715a0c5e8bf0354ea2dd58fc`.

De echte ring-3-systeemapp gebruikt nu `Session<NativeTransport>` voor STATUS,
Generate, legacy Seal/Unseal, contextgebonden SealRecord/UnsealRecord en Revoke.
De client bezit zijn twee bestaande kanalen en sluit die pas na alle proeven.
De vertrouwde negatieve testfixture leent de numerieke handles uitsluitend
tussen volledig afgeronde SDK-aanvragen. Alle bestaande werkelijke negatieve
IPC-proeven blijven behouden, inclusief malformed contextbytes, contextwijziging,
beschadigde ciphertext en intrekking. Zij worden door de dienst geweigerd en
niet vervangen door lokale SDK-validatie. Er zijn geen nieuwe kernelrechten.

De vastgelegde native gate voerde twee VMs uit: normale start (3,94 s) en koude
read-only-herstart (4,31 s), beide raw exit 33 en alle verplichte markers. Beide
rapporten noemen exact de bovenstaande broncommit. De cryptotellers zijn
exact `seal=2`, `unseal=2`, `tamper_denied=9`; de recordtellers zijn exact
`1/1/10/2`. De app-ELF is in beide runs dezelfde:
`339f8a8748d23111c5c8c87fc10d78947a9a71cd664abcf2367534ad21d71bb2`.

De onafhankelijke schijforacle controleerde naam, inhoud, revisie en overige
objecten van het bestaande 4096-byte-document. De koude run gebruikte exact
het geschreven image en wijzigde geen byte:
`ad40c83d6b9d0a409391113f7ed4deb5aafd7c27f6629c22d7a6ddc060c639ea`.
Verder slagen 34 systeemharnastests, 17 SDK-runtimechecks, Clippy met warnings
als fouten en de formattercontrole.

Uitgevoerde hoofdcommando's vanuit deze worktree:

```sh
LOCALAPPDATA=/workspace/nieuw-os/.tools PATH="/workspace/.nieuw-os-env/bin:$PATH" \
  OS_OVMF_CODE=/workspace/.nieuw-os-env/sysroot/usr/share/OVMF/OVMF_CODE_4M.fd \
  OS_OVMF_VARS=/workspace/.nieuw-os-env/sysroot/usr/share/OVMF/OVMF_VARS_4M.fd \
  NOS_QEMU_MACHINE=pc-q35-8.2 NOS_BUILD_DIR="$PWD/build/keyring-sdk-native" \
  NOS_TARGET_DIR="$PWD/target/keyring-sdk-native-gate" \
  CARGO_TARGET_DIR="$PWD/target/keyring-sdk-native-gate" \
  python3 -B -u tools/test_system.py
LOCALAPPDATA=/workspace/nieuw-os/.tools NOS_BUILD_DIR="$PWD/build/keyring-sdk-native" \
  python3 -B tools/capture_system.py \
  --destination docs/verification/full-release-keyring-sdk-fd7d9a0
```

`capture_system.py` las de echte images en ESP-binaries opnieuw, controleerde
de bytes tegen manifests, de raw serialtellers en de koude imageketen, en
archiveerde alleen tekst/hashbewijs. De kernels, app-ELF en schijfimages blijven
in de geïsoleerde build-/targetmappen. De uiteindelijke normale kernel is
het werkelijk geteste artifact, geen teruggebouwde bootdemo.

Dit vult de eerdere nog niet uitgevoerde native-clientpoort uit
`KEYRING-SDK-CLIENT.md` voor dit profiel in. Het bewijst **geen** duurzaam
sleutelherstel of versleutelde documenten op de schijf: het documentfixture is
plaintext, de recordproeven werken in geheugen en de sleuteldienst start in
iedere VM opnieuw met verse entropy. Object-/actieautorisatie door de
productbroker, persistente sleutels/snapshots, rollbackbescherming en fysieke
hardware blijven open. Dit is geen volledige releaseacceptatie.
