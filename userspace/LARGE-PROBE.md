# Targeted M3 regression: more than 64 user mappings

`large_linker.ld` links the same unmodified `probe.rs` with 33 extra NOLOAD BSS pages. Entry/text and the four data words stay the same. The existing `linker.ld`, M3 probe and M3 tests remain unchanged.

```powershell
python tools/build_users.py --kind probe
python tools/build_users.py --kind large
python -B -X utf8 userspace/tests/test_large_probe.py --elf build/users/probe_large.elf
```

Root owns the build tool and the real `test-m3-large-image` QEMU run. Output: `build/users/probe_large.elf` and `build/users/manifest-large.json`. The selected variant is embedded as `NOS_USER_ELF`. No new Rust configuration, library or user code is needed.

Exact accounting, agreed with the kernel agent:

- RX at `0x40000000`, one page; the file bytes are identical to normal `probe.elf`.
- RW at `0x40010000`, 32 file bytes and 139264 memory bytes: 34 pages.
- `.bss` starts at `0x40011000`, is SHT_NOBITS/NOLOAD and spans 33×4096 bytes. The bytes between the four data words and the next page boundary likewise belong to the segment memory to be zeroed.
- Image: 35 pages. With 16 heap and 16 stack pages: **67 user mappings**, above the old64limit and below the contract limit2048.

Six host tests confirm these artifact properties, identical text bytes, source/linker/binary hashes and rejection of a mutated unsafe segment. This does not prove actually present/zeroed guest RAM or a successful process start. The real kernel run must create and audit all67 mappings, zero the large BSS and actually execute the existing user probe.

Evidence: `verification/U03-LARGE-HOST-2026-10-27.md`, `verification/large-host-evidence.json` and the actual test log. The binary resides exclusively in the ignored build path. Recovery: remove this optional linker/host test and have root revert the variant binding; normal M3 and the IPC fixture remain usable.
