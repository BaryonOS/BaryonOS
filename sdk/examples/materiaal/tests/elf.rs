#![cfg(feature = "verify-native-elf")]
//! Parse the explicitly supplied actual executable with the same loader policy
//! as the kernel. Absence is a failure, never an automatically skipped pass.
#[test]
fn actual_native_elf_has_five_bounded_segments_and_private_bss_buffers() {
    let path = std::env::var_os("NOS_MATERIAAL_ELF")
        .expect("set NOS_MATERIAAL_ELF to the actual built executable");
    let bytes = std::fs::read(&path).expect("read actual executable");
    let image = kernel_core::user::parse(&bytes).expect("kernel user::parse accepts actual ELF");
    assert_eq!(image.entry, 0x40000000);
    assert_eq!(image.count, 5);
    let expected = [
        (0x40000000, false, true),
        (0x40010000, true, false),
        (0x40020000, false, false),
        (0x400b0000, true, false),
        (0x400f0000, false, true),
    ];
    for (segment, (va, writable, executable)) in image.segments[..5].iter().zip(expected) {
        assert_eq!(
            (segment.va, segment.writable, segment.executable),
            (va, writable, executable)
        );
        assert!(!(segment.writable && segment.executable));
    }
    assert!(image.segments[0].memory_size <= 4096);
    let entry = image.segments[0].file_offset as usize;
    assert_eq!(
        &bytes[entry..entry + 10],
        &[0xfc, 0x48, 0x89, 0xe0, 0x83, 0xe0, 0x0f, 0x83, 0xf8, 0x08],
        "actual CLD/MOV/AND/CMP C-entry guard"
    );
    let entrycode = std::process::Command::new("objdump")
        .args(["-d", "--no-show-raw-insn", "--disassemble=_start"])
        .arg(&path)
        .output()
        .unwrap();
    assert!(entrycode.status.success());
    let entrycode = std::str::from_utf8(&entrycode.stdout).unwrap();
    assert!(entrycode
        .lines()
        .any(|line| line.contains("jmp") && line.contains("<materiaal_main>")));
    assert!(
        !entrycode.lines().any(|line| line.contains("call")),
        "entry must tail-jump without extra return address"
    );
    assert_eq!(image.segments[1].memory_size, 1024);
    assert!(image.segments[2].memory_size <= 512 * 1024);
    let data = image.segments[3];
    assert!(data.memory_size >= 3 * 65536 && data.memory_size <= 256 * 1024);
    assert!(data.memory_size - data.file_size >= 3 * 65536);
    assert!(image.segments[4].memory_size <= 160 * 1024);
    assert!(image.pages + 32 <= kernel_core::USER_QUOTA);
    // Inspect actual symbols to bind the BSS storage, rather than inferring it
    // merely from enough blank space in a writable segment.
    let symbols = std::process::Command::new("nm")
        .arg("-S")
        .arg(&path)
        .output()
        .expect("nm available");
    assert!(symbols.status.success());
    let symbols = std::str::from_utf8(&symbols.stdout).unwrap();
    let line = symbols
        .lines()
        .find(|line| line.ends_with(" materiaal_document_buffers"))
        .expect("buffer symbol exists");
    let fields: Vec<_> = line.split_whitespace().collect();
    let address = u64::from_str_radix(fields[0], 16).unwrap();
    let size = u64::from_str_radix(fields[1], 16).unwrap();
    assert_eq!(fields[2], "B");
    assert_eq!(size, 3 * 65536);
    assert!(address >= data.va + data.file_size && address + size <= data.va + data.memory_size);
    assert_eq!(address % 4096, 0);
    println!(
        "actual ELF: {} pages, buffers {address:#x}+{size:#x}, five W^X LOAD segments",
        image.pages
    );
}
