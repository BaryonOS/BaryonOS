# B01 example: Key Service and acceptance app

Two ordinary `no_std` programs for Baryon OS ABI 1.4 (`native-keyring` profile,
ADR-043). Policy and all key material reside in the
[`keyring-core`](../../../services/keyring/) library; this example only moves
bytes between the kernel broker, IPC envelopes and that core.

| Program | Startup authority | Behaviour |
|---|---|---|
| `keyring-service` | one `Entropy` capability, two domain ingress endpoints, an optional separate manifest-bound management ingress, and one `SEND` to the app | obtains 1024 bytes of startup entropy through the kernel broker, runs SP 800-90B health tests and becomes READY or BLOCKED; domain ingress serves STATUS, GENERATE, SEAL, UNSEAL, REVOKE and record operations; only the management ingress may perform EXPORT_RECOVERY, RESTORE and persistent key-state operations |
| `keyring-app` | `SEND` to both domains of two service instances and one owned reply endpoint; no management capability | drives generation, sealing, opening, revocation and quota; tests tampering, wrong domain, revoked keys, missing entropy and explicit denial of recovery export/import; every observation is an evidence word checked by the kernel |

Cryptography comes from pinned RustCrypto crates; the build (`build.py`)
disables all x86 SIMD backends because the ring-3 profile has no SSE/AVX
(`chacha20_force_soft`, `chacha20_backend="soft"`,
`poly1305_backend="soft"`, `sha2_backend="soft"`).
The separate linker script gives code 128 KiB from `0x40020000`;
the startup page remains at `0x40010000`.

```sh
python3 -B sdk/examples/keyring/build.py
```

Default output: `build/sdk-keyring/x86_64-unknown-none/release/keyring-*`.
The kernel gate `tools/test_keyring.py` builds these binaries itself through
`tools/build.py --kernel-feature native-keyring` and the two test variants.

## Limits

No persistent key storage, hardware protection of the master key or password
recovery is provided. The app is an acceptance fixture, not a product
key-management interface; recovery management requires the separate,
manifest-authorised management ingress. The service is the first native Key
Service boundary for B02, B03, N03 and P01.
