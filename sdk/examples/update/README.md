# B03 example: update service

One ordinary `no_std` program for the Baryon OS ABI (`native-update` test
profile, ADR-049). Fixture policy is explicitly opt-in through Cargo feature
`test-signing-anchors`; without that feature, the service does not compile.
All format and policy knowledge resides in
[`update-core`](../../../crates/update-core/); this example only moves
sectors between the kernel broker and that core.

| Program | Startup authority | Behaviour |
|---|---|---|
| `update-service` | one block capability over the entire slot storage (read, write, flush) | reads the header and active manifest, verifies the inbox bundle with both TEST anchors and the image digest **before** any write, uses `update_core::decide` against the current version reported by the kernel, installs in the inactive slot (image, FLUSH, manifest, FLUSH, header, FLUSH) and reports everything as evidence words |

The build (`build.py`) explicitly enables this test feature and pins the
portable curve25519 backend; `update-core` pins sha2 to `force-soft`.
The separate linker script gives code 256 KiB from `0x40030000`;
the startup page remains at `0x40010000`.

```sh
python3 -B sdk/examples/update/build.py
```

Default output: `build/sdk-update/x86_64-unknown-none/release/update-service`.
The kernel gate `tools/test_update.py` builds this binary itself through
`tools/build.py --kernel-feature native-update`.

## Limits

No network, package or app updates (A02), release keys (anchors are TEST anchors)
or rollback-floor management outside the loader: the service knows only the
current kernel version and rejects every version at or below it.
