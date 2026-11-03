# Herbouw kernelprofielen na resetpadfix

**Status:** gerichte freestanding kernelbuilds geslaagd; geen runtime- of
hardwareacceptatie.

- Geteste bron: `449ecd654d5a4c15f69c8d3038a59369366adba6`
- Branch: `copilot/universal-driver-framework`
- Remote branch gecontroleerd op dezelfde SHA.
- PR #153: open; head `449ecd654d5a4c15f69c8d3038a59369366adba6`,
  base `5c66ec57aaf8eb411a83b8d6e42320ddc379383c`.
- Hostcompiler: tijdelijk uitgepakt onder
  `/tmp/nieuw-os-hostcc-2026-11-03/`; geen systeeminstallatie of
  repositorydependency gewijzigd.

Alle zes profielbuilds gebruikten dezelfde compileromgeving:

```sh
env \
  PATH="/tmp/nieuw-os-hostcc-2026-11-03/bin:/home/simon/.cargo/bin:$PATH" \
  LD_LIBRARY_PATH=/tmp/nieuw-os-hostcc-2026-11-03/root/usr/lib/x86_64-linux-gnu \
  GCC_EXEC_PREFIX=/tmp/nieuw-os-hostcc-2026-11-03/root/usr/lib/gcc/x86_64-linux-gnu/15/ \
  COMPILER_PATH=/tmp/nieuw-os-hostcc-2026-11-03/root/usr/libexec/gcc/x86_64-linux-gnu/15 \
  LIBRARY_PATH=/tmp/nieuw-os-hostcc-2026-11-03/linker-script:/tmp/nieuw-os-hostcc-2026-11-03/root/usr/lib/x86_64-linux-gnu:/tmp/nieuw-os-hostcc-2026-11-03/root/usr/lib/gcc/x86_64-linux-gnu/15 \
  CC=/tmp/nieuw-os-hostcc-2026-11-03/bin/cc \
  python3 -B tools/build.py --kernel-feature <profiel>
```

| Profiel | Exacte profielwaarde | Exit | Duur | Log | SHA-256 |
|---|---|---:|---:|---|---|
| M5 | `milestone-m5` | 0 | 1,30 s | `/tmp/nieuw-os-kernel-profile-retest-2026-11-03/m5.log` | `1b298f8b0cbf85cbc8ed2c2107df8ad2647171aa77f9534af9262bc89d51aa0b` |
| M6a | `milestone-m6a` | 0 | 1,61 s | `/tmp/nieuw-os-kernel-profile-retest-2026-11-03/m6a.log` | `75fd14f6979c58e83fb8dd8d65853c9ab964d6043dd76496b9dbdb44779e6ceb` |
| M6b | `milestone-m6b` | 0 | 1,61 s | `/tmp/nieuw-os-kernel-profile-retest-2026-11-03/m6b.log` | `dac3e2767ee96ff5b37fac5cb6b65f5453e535637540805a55a463f06bd97a97` |
| M6c | `milestone-m6c` | 0 | 1,65 s | `/tmp/nieuw-os-kernel-profile-retest-2026-11-03/m6c.log` | `4a8c9beb2b05c71840829fcb7628eb0c78d5a2defb7df609d90cc88a783be063` |
| Volume | `native-volume` | 0 | 1,07 s | `/tmp/nieuw-os-kernel-profile-retest-2026-11-03/volume.log` | `6e766d2bc71bb25252cc12040082c87c9d55dda4ddeabb7346c746ff283eb7ec` |
| Update | `native-update --image-version 1` | 0 | 7,30 s | `/tmp/nieuw-os-kernel-profile-retest-2026-11-03/update.log` | `0e31d6962c6c414801cffb0482900eda6303d5d5de1cf55d2fcbbc27dc344e61` |

De updateversie `1` is uitsluitend een tijdelijke compile-invoer, geen
releaseversie. Deze builds bevestigen dat de eerdere `as_ref()`/`&mut self`
typefout is verholpen. Ze bewijzen geen runtimegedrag, apparaatdetectie,
DMA-isolatie of fysieke hardwareondersteuning. QEMU-proeven en fysieke
hardwaretests zijn niet uitgevoerd. De zes block-profielen matchen PCI vóór
DMA-poolallocatie; overige resourcegrants en driverfactory-activatie ontbreken
nog.
