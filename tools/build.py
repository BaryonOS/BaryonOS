#!/usr/bin/env python3
"""Build the actual UEFI loader and ELF kernel; inspect artifacts and prepare ESP."""
from __future__ import annotations
import argparse
import json
import os
import shutil
import struct
import sys
from common import ROOT, BUILD, TARGET, run_logged, save_json, sha256, source_hash
from gate_contract import M3_CASES, M4_CASES
from m6c_faults import FAULT_CASES as M6C_FAULT_CASES
M5_CASES = ("service-errors", "device-rights", "async", "bad-descriptor", "used-id",
            "used-index", "used-short", "used-long", "used-status", "io-error", "timeout",
            "reset-timeout", "restart", "cpu-mmio", "cpu-dma", "cpu-pci", "no-device")
M6A_CASES = ("service-errors", "device-rights", "full", "write-error", "flush-error",
             "write-timeout", "flush-timeout", "reset-timeout", "no-flush", "readonly",
             "bad-descriptor", "used-write", "used-flush", "torn-tail", "corrupt-all",
             "stale-generation", "no-device", *(f"service-cut-{i}" for i in range(1, 7)),
             *(f"vm-cut-{i}" for i in range(1, 7)), "recover")
M6B_TIMEOUT_CASES = ("write-timeout", "flush-timeout", "reset-timeout", "write-poll-limit", "flush-poll-limit")
M6B_FAULT_CASES = ("write-error", "flush-error") + M6B_TIMEOUT_CASES
M6B_CASES = ("recover", "errors", "full", "corrupt", "vm-cut-before-ack", "vm-cut-after-flush") + M6B_FAULT_CASES
M6C_CASES = ("recover", "errors", "full", "corrupt", "legacy", "vm-cut-before-rename", "vm-cut-after-rename") + M6C_FAULT_CASES
VOLUME_CASES = ("recover", "cut-data", "cut-table", "recover-cut", "corrupt", "foreign", "cut-txn")
VOLUME_FEATURES = ("native-volume", *(f"test-volume-{case}" for case in VOLUME_CASES))
SUPERVISOR_CASES = ("crash", "denied", "parent-exit", "quota", "wait")
SUPERVISOR_FEATURES = ("native-supervisor", *(f"test-supervisor-{case}" for case in SUPERVISOR_CASES))
KEYRING_CASES = ("no-device", "stuck", "timeout", "restart")
KEYRING_FEATURES = ("native-keyring", *(f"test-keyring-{case}" for case in KEYRING_CASES))
UPDATE_FEATURES = ("native-update", "test-update-cut")
NETWORK_SYSTEM_FEATURES = ("native-system-net", "test-system-net-recover")
TCP_SYSTEM_FEATURES = ("native-system-tcp", "test-system-tcp-recover")
MEMORY_FEATURES = ("native-memory", "native-memory-nx", "native-memory-guard", "native-memory-peer", "native-memory-oom")
WINDOW_FEATURES = ("native-window-session", "native-window-session-probe", "native-window-session-hidpi", "native-session-route")
MATERIAAL_FEATURES = (*WINDOW_FEATURES, "native-system-materiaal", "native-system-session-preview", "native-system-materiaal-probe", "test-system-materiaal-recover", "test-system-materiaal-scale200")
NAS_FEATURES = ("native-system-nas-probe", "test-system-nas-recover")
RANDOM_FEATURES = ("native-system-random", "test-system-random-sdk", "test-system-random-unknown", "test-system-random-wrong-caller")
IMAGE_FEATURES = ("native-system-images",)
SYSTEM_CLOCK_FEATURES = ("native-system-clock",)
SYSTEM_FEATURES = ("native-net-service", "test-net-service-client-exit", "native-cap-digest-bench", "native-system", "test-system-recover", "native-system-display-input", *NETWORK_SYSTEM_FEATURES, *TCP_SYSTEM_FEATURES, *RANDOM_FEATURES, *IMAGE_FEATURES, *SYSTEM_CLOCK_FEATURES)
KERNEL_FEATURES = ["test-panic", "test-hang", "milestone-m2", "native-rtc",
                   "native-acpi-s5", "test-acpi-s5-corrupt-dsdt",
                   "native-smp", "test-smp-ap-stall", "test-smp-ap-mismatch", "test-smp-ap-nmi",
                   "test-smp-ap-double-fault", "test-smp-ap-ipi-deaf", "test-smp-ap-shootdown-deaf",
                   "test-smp-shootdown-skipped", "test-smp-ap-timer-dead", "test-m2-ro", "test-m2-nx",
                   "test-m2-absent", "test-m2-kernel-fault", "test-m2-double-fault",
                   "milestone-m3", "test-m3-idle", "test-m3-large-image", *(f"test-m3-{case}" for case in M3_CASES),
                   "milestone-m4", *(f"test-m4-{case}" for case, _ in M4_CASES),
                   "milestone-m5", *(f"test-m5-{case}" for case in M5_CASES),
                   "milestone-m6a", *(f"test-m6a-{case}" for case in M6A_CASES),
                   "milestone-m6b", *(f"test-m6b-{case}" for case in M6B_CASES),
                   "milestone-m6c", *(f"test-m6c-{case}" for case in M6C_CASES), "graphics-g0", "native-qsim", "native-qsim-job",
                   "native-sdk-app", "test-sdk-app-denied", "native-ipc-wait",
                   "native-tpm-crb", "test-tpm-crb-absent", *VOLUME_FEATURES, *SUPERVISOR_FEATURES,
                   *KEYRING_FEATURES, *MATERIAAL_FEATURES, *NAS_FEATURES, *UPDATE_FEATURES, *SYSTEM_FEATURES, *MEMORY_FEATURES, "native-fpu", "test-fpu-wait", "framebuffer", "native-display-probe", "native-system-display-input"]


def inspect_elf(path):
    data = path.read_bytes()
    if len(data) < 64 or data[:7] != b"\x7fELF\x02\x01\x01":
        raise ValueError("not ELF64 little endian")
    kind, machine, version, entry, phoff, shoff, flags, ehsize, phsize, phnum, shsize, shnum, shstr = struct.unpack_from("<HHIQQQIHHHHHH", data, 16)
    if (kind, machine, version, ehsize, phsize) != (2, 62, 1, 64, 56) or not 1 <= phnum <= 64:
        raise ValueError("invalid ELF header")
    segments = []
    if phoff + phsize * phnum > len(data):
        raise ValueError("truncated program header table")
    for n in range(phnum):
        typ, fl, off, virt, phys, filesz, memsz, align = struct.unpack_from("<IIQQQQQQ", data, phoff + n * phsize)
        if typ not in (0, 1, 4, 6, 0x6474e551) or (typ == 0x6474e551 and fl & 1):
            raise ValueError("unsupported or executable metadata segment")
        if typ != 1 or memsz == 0:
            continue
        if not (filesz <= memsz and off + filesz <= len(data) and phys == virt and 0x2000000 <= virt < virt + memsz <= 0x3000000 and fl in (4, 5, 6) and align >= 4096 and align & (align-1) == 0 and off % align == virt % align):
            raise ValueError("invalid load segment")
        low, high = virt // 4096 * 4096, (virt + memsz + 4095) // 4096 * 4096
        if any(low < old["page_end"] and old["page_start"] < high for old in segments):
            raise ValueError("overlapping segment pages")
        segments.append(dict(flags=fl, offset=off, address=virt, filesz=filesz, memsz=memsz, page_start=low, page_end=high))
    if sum(s["flags"] == 5 and s["address"] <= entry < s["address"] + s["filesz"] for s in segments) != 1:
        raise ValueError("entry not in exactly one loaded RX segment")
    if shnum == 0 or shsize != 64 or shoff + shnum * shsize > len(data):
        raise ValueError("missing/invalid build-time section table")
    section_headers = [struct.unpack_from("<IIQQQQIIQQ", data, shoff + n * shsize) for n in range(shnum)]
    if shstr >= shnum:
        raise ValueError("invalid section name table")
    names_head = section_headers[shstr]
    names = data[names_head[4]:names_head[4]+names_head[5]]
    sections = []
    for section in section_headers:
        name = names[section[0]:].split(b"\0", 1)[0].decode("ascii", "strict")
        if section[2] & 2:
            if name not in (".text", ".rodata", ".data", ".bss"):
                raise ValueError(f"unexpected allocatable section {name}")
            if not any(s["address"] <= section[3] and section[3] + section[5] <= s["address"] + s["memsz"] for s in segments):
                raise ValueError(f"section outside load plan: {name}")
        if section[1] in (4, 6, 9, 11) and section[5]:
            raise ValueError(f"dynamic/relocation metadata: {name}")
        sections.append(dict(name=name, type=section[1], flags=section[2], size=section[5]))
    return dict(entry=entry, segments=segments, sections=sections)


def inspect_pe(path):
    data = path.read_bytes()
    if data[:2] != b"MZ" or len(data) < 64:
        raise ValueError("loader is not PE")
    pe = struct.unpack_from("<I", data, 60)[0]
    if data[pe:pe+4] != b"PE\0\0" or struct.unpack_from("<H", data, pe+4)[0] != 0x8664:
        raise ValueError("loader is not AMD64 PE")
    opt = pe+24
    if struct.unpack_from("<H", data, opt)[0] != 0x20b or struct.unpack_from("<H", data, opt+68)[0] != 10:
        raise ValueError("loader is not PE32+ EFI application")
    if struct.unpack_from("<II", data, opt+120) != (0, 0):
        raise ValueError("loader unexpectedly imports host DLLs")
    return {"format": "AMD64 PE32+", "subsystem": "EFI_APPLICATION", "imports": 0}


def build(loader_feature=None, kernel_feature=None, label="normal", image_version=None):
    if kernel_feature in WINDOW_FEATURES:
        wanted="desktop-hidpi" if kernel_feature=="native-window-session-hidpi" else "desktop-mode"
        if loader_feature not in (None,wanted):
            raise ValueError("Window session requires its matching desktop loader")
        loader_feature=wanted
    elif kernel_feature == "graphics-g0":
        if loader_feature not in (None, "graphics-g0"):
            raise ValueError("G0 requires its paired graphics loader")
        loader_feature = "graphics-g0"
    elif kernel_feature in ("framebuffer", "native-system-display-input", "native-display-probe", *MATERIAAL_FEATURES):
        if loader_feature not in (None, "framebuffer"):
            raise ValueError("Framebuffer transport requires its paired loader")
        loader_feature = "framebuffer"
    elif loader_feature in ("graphics-g0", "framebuffer", "desktop-mode", "desktop-hidpi"):
        raise ValueError("Graphics loader requires a matching framebuffer kernel")
    if kernel_feature in UPDATE_FEATURES:
        # B03: these update kernels are test profiles with explicit TEST anchors.
        if loader_feature not in (None, "signed-boot-test"):
            raise ValueError("update test kernels require the signed-boot-test loader")
        loader_feature = "signed-boot-test"
        if image_version is None or not 1 <= image_version <= 2**32:
            raise ValueError("update kernels need an explicit --image-version")
    elif loader_feature == "signed-boot":
        raise ValueError("signed-boot has no configured policy; TEST fixtures require signed-boot-test")
    elif image_version is not None:
        raise ValueError("--image-version applies to update kernels only")
    extra_env = None
    sdk_selection = os.environ.get("NOS_SDK_RUNTIME", "0")
    if sdk_selection not in ("0", "1"):
        raise ValueError("NOS_SDK_RUNTIME must be exactly 0 or 1")
    use_sdk = sdk_selection == "1" and kernel_feature is not None and "m6c" in kernel_feature
    document_selection = os.environ.get("NOS_DOCUMENT_SESSION", "0")
    if document_selection not in ("0", "1"):
        raise ValueError("NOS_DOCUMENT_SESSION must be exactly 0 or 1")
    use_document_session = document_selection == "1"
    if use_document_session and (not use_sdk or kernel_feature not in
                                ("milestone-m6c", "test-m6c-recover")):
        raise ValueError("Document session requires NOS_SDK_RUNTIME=1 and the M6c normal/recover fixture")
    qsim_report = None
    if kernel_feature in NAS_FEATURES:
        from build_nas import build_users as build_nas_users
        extra_env, qsim_report = build_nas_users()
    elif kernel_feature in MATERIAAL_FEATURES:
        from build_materiaal import build_users as build_materiaal_users
        extra_env, qsim_report = build_materiaal_users(probe=kernel_feature not in ("native-system-materiaal", "native-system-session-preview", *WINDOW_FEATURES), window_session=kernel_feature in WINDOW_FEATURES, fault_injection=kernel_feature=="native-window-session-probe", session_preview=kernel_feature=="native-session-route")
    elif kernel_feature == "native-display-probe":
        from run_display_probe import build_users as build_display_users
        extra_env, qsim_report = build_display_users()
    elif kernel_feature in MEMORY_FEATURES:
        from run_memory import build_users as build_memory_users
        extra_env, qsim_report = build_memory_users()
    elif kernel_feature in ("native-fpu", "test-fpu-wait"):
        from run_fpu import build_users as build_fpu_users
        extra_env, qsim_report = build_fpu_users()
    elif kernel_feature == "native-qsim":
        output = BUILD / "users/qsim"
        run_logged([sys.executable, "-B", str(ROOT / "experiments/quantum/native/build.py"),
                    "--output-dir", str(output), "--target-dir", str(TARGET / "qsim-user")],
                   "build-qsim-user")
        qsim_report = json.loads((output / "build-report.json").read_text())
        if not qsim_report.get("passed") or sha256(output / "qsim-native-probe.elf") != qsim_report["binary_sha256"]:
            raise ValueError("Native qsim executable failed its artifact gate")
        extra_env = {"NOS_QSIM_ELF": str((output / "qsim-native-probe.elf").resolve())}
    elif kernel_feature == "native-qsim-job":
        output = BUILD / "users/qsim-job"
        run_logged([sys.executable, "-B", str(ROOT / "experiments/quantum/native/job/build.py"),
                    "--output-dir", str(output), "--target-dir", str(TARGET / "qsim-job-user")],
                   "build-qsim-job-user")
        qsim_report = json.loads((output / "build-report.json").read_text())
        if not qsim_report.get("passed") or sha256(output / "qsim-job-native.elf") != qsim_report["binary_sha256"]:
            raise ValueError("Native qsim job executable failed its artifact gate")
        extra_env = {"NOS_QSIM_JOB_ELF": str((output / "qsim-job-native.elf").resolve())}
    elif kernel_feature in ("native-ipc-wait", "native-tpm-crb", "test-tpm-crb-absent"):
        from build_users import inspect_user
        app_target = TARGET / "ipc-wait-app"
        run_logged([sys.executable, "-B", str(ROOT / "sdk/examples/ipc-wait-probe/build.py")],
                   "build-ipc-wait-app", extra_env={"CARGO_TARGET_DIR": str(app_target)})
        app = app_target / "x86_64-unknown-none/release/ipc-wait-probe"
        qsim_report = {"app_sha256": sha256(app), "app_elf": inspect_user(app)}
        extra_env = {"NOS_SDK_APP_ELF": str(app.resolve()),
                     "NOS_SDK_SERVICE_ELF": str(app.resolve())}
    elif kernel_feature in ("native-sdk-app", "test-sdk-app-denied"):
        from build_users import inspect_user
        app_target = TARGET / "sdk-app"
        run_logged([sys.executable, "-B", str(ROOT / "sdk/examples/catalog-read/build.py")],
                   "build-sdk-example", extra_env={"CARGO_TARGET_DIR": str(app_target)})
        app = app_target / "x86_64-unknown-none/release/catalog-read"
        service = app_target / "x86_64-unknown-none/release/catalog-read-service"
        qsim_report = {
            "app_sha256": sha256(app), "app_elf": inspect_user(app),
            "service_sha256": sha256(service), "service_elf": inspect_user(service),
        }
        extra_env = {
            "NOS_SDK_APP_ELF": str(app.resolve()),
            "NOS_SDK_SERVICE_ELF": str(service.resolve()),
        }
    elif kernel_feature in SYSTEM_FEATURES:
        from build_users import inspect_user
        binaries = {}
        # These are ordinary freestanding ring-3 programs. Their existing
        # builders keep each linker/portable-crypto contract and an isolated
        # target directory; the kernel receives only the inspected ELFs.
        for component, package, names in (
            ("system", "system", ("init", "app", "worker")),
            ("keyring", "keyring", ("service",)),
            ("volume", "volume-docs", ("service",)),
        ):
            component_target = TARGET / f"system-{component}"
            command = [sys.executable, "-B", str(ROOT / f"sdk/examples/{package}/build.py")]
            if package == "system" and kernel_feature in SYSTEM_CLOCK_FEATURES:
                command.append("--native-clock")
            run_logged(command,
                       f"build-system-{component}",
                       extra_env={"CARGO_TARGET_DIR": str(component_target)})
            for name in names:
                binary = component_target / f"x86_64-unknown-none/release/{component}-{name}"
                binaries[f"{component}_{name}"] = binary
        if kernel_feature in NETWORK_SYSTEM_FEATURES + TCP_SYSTEM_FEATURES:
            probe = "tcp-probe" if kernel_feature in TCP_SYSTEM_FEATURES else "net-probe"
            net_target = TARGET / f"system-{probe}"
            run_logged([sys.executable, "-B", str(ROOT / f"sdk/examples/{probe}/build.py")],
                       "build-system-net-probe", extra_env={"CARGO_TARGET_DIR": str(net_target)})
            binaries["net_probe"] = net_target / f"x86_64-unknown-none/release/{probe}"
        if kernel_feature in ("native-net-service", "test-net-service-client-exit"):
            net_target = TARGET / "network-service"
            run_logged([sys.executable, "-B", str(ROOT / "sdk/examples/network-service/build.py")],
                       "build-network-service", extra_env={"CARGO_TARGET_DIR": str(net_target)})
            private = BUILD / "users/network-service"
            private.mkdir(parents=True, exist_ok=True)
            for name in ("service", "client"):
                source = net_target / f"x86_64-unknown-none/release/network-{name}"
                binary = private / f"network-{name}.elf"
                if binary.exists() or binary.is_symlink():
                    if binary.is_symlink() or not binary.is_file() or binary.stat().st_nlink != 1:
                        raise ValueError("linked or special network ELF output")
                    binary.unlink()
                shutil.copy2(source, binary)
                binaries[f"network_{name}"] = binary
        if kernel_feature in RANDOM_FEATURES + IMAGE_FEATURES:
            package="random-probe" if kernel_feature in RANDOM_FEATURES else "image-probe"
            probe_target=TARGET / ("random-probe-app" if package=="random-probe" else "system-image-probe")
            run_logged([sys.executable,"-B",str(ROOT / f"sdk/examples/{package}/build.py")],"build-system-"+package,extra_env={"CARGO_TARGET_DIR":str(probe_target)})
            binaries[package.replace("-","_")]=probe_target / f"x86_64-unknown-none/release/{package}"
        qsim_report = {name: {"sha256": sha256(path), "elf": inspect_user(path)}
                       for name, path in binaries.items()}
        extra_env = {f"NOS_{name.upper()}_ELF": str(path.resolve())
                     for name, path in binaries.items()}
    elif kernel_feature in VOLUME_FEATURES:
        from build_users import inspect_user
        volume_target = TARGET / "volume-docs"
        run_logged([sys.executable, "-B", str(ROOT / "sdk/examples/volume-docs/build.py")],
                   "build-volume-docs", extra_env={"CARGO_TARGET_DIR": str(volume_target)})
        app = volume_target / "x86_64-unknown-none/release/volume-app"
        service = volume_target / "x86_64-unknown-none/release/volume-service"
        qsim_report = {
            "app_sha256": sha256(app), "app_elf": inspect_user(app),
            "service_sha256": sha256(service), "service_elf": inspect_user(service),
        }
        extra_env = {
            "NOS_VOLUME_APP_ELF": str(app.resolve()),
            "NOS_VOLUME_SERVICE_ELF": str(service.resolve()),
        }
    elif kernel_feature in SUPERVISOR_FEATURES:
        from build_users import inspect_user
        supervisor_target = TARGET / "supervisor"
        run_logged([sys.executable, "-B", str(ROOT / "sdk/examples/supervisor/build.py")],
                   "build-supervisor", extra_env={"CARGO_TARGET_DIR": str(supervisor_target)})
        binaries = {name: supervisor_target / f"x86_64-unknown-none/release/k05-{name}"
                    for name in ("supervisor", "worker", "client")}
        qsim_report = {f"{name}_sha256": sha256(path) for name, path in binaries.items()}
        qsim_report.update({f"{name}_elf": inspect_user(path) for name, path in binaries.items()})
        extra_env = {f"NOS_{name.upper()}_ELF": str(path.resolve()) for name, path in binaries.items()}
    elif kernel_feature in UPDATE_FEATURES:
        from build_users import inspect_user
        update_target = TARGET / "update"
        run_logged([sys.executable, "-B", str(ROOT / "sdk/examples/update/build.py")],
                   "build-update-service", extra_env={"CARGO_TARGET_DIR": str(update_target)})
        service = update_target / "x86_64-unknown-none/release/update-service"
        qsim_report = {"service_sha256": sha256(service), "service_elf": inspect_user(service),
                       "image_version": image_version}
        extra_env = {"NOS_UPDATE_SERVICE_ELF": str(service.resolve()),
                     "NOS_IMAGE_VERSION": str(image_version)}
    elif kernel_feature in KEYRING_FEATURES:
        from build_users import inspect_user
        keyring_target = TARGET / "keyring"
        run_logged([sys.executable, "-B", str(ROOT / "sdk/examples/keyring/build.py")],
                   "build-keyring", extra_env={"CARGO_TARGET_DIR": str(keyring_target)})
        binaries = {name: keyring_target / f"x86_64-unknown-none/release/keyring-{name}"
                    for name in ("service", "app")}
        qsim_report = {f"{name}_sha256": sha256(path) for name, path in binaries.items()}
        qsim_report.update({f"{name}_elf": inspect_user(path) for name, path in binaries.items()})
        extra_env = {f"NOS_KEYRING_{name.upper()}_ELF": str(path.resolve()) for name, path in binaries.items()}
    if kernel_feature and any(m in kernel_feature for m in ("m3", "m4", "m5", "m6a", "m6b", "m6c")):
        from build_users import build_users
        extra_env = {"NOS_USER_ELF": str(build_users("large" if kernel_feature == "test-m3-large-image" else "probe").resolve())}
        if any(m in kernel_feature for m in ("m4", "m5", "m6a", "m6b", "m6c")):
            extra_env["NOS_IPC_ELF"] = str(build_users("ipc").resolve())
        if any(m in kernel_feature for m in ("m5", "m6a", "m6b", "m6c")):
            extra_env["NOS_M5_ELF"] = str(build_users("m5").resolve())
        if "m6a" in kernel_feature:
            extra_env["NOS_M6_ELF"] = str(build_users("m6").resolve())
        if "m6b" in kernel_feature:
            extra_env["NOS_M6B_ELF"] = str(build_users("m6b", short_poll=kernel_feature in
                ("test-m6b-write-poll-limit", "test-m6b-flush-poll-limit")).resolve())
        if "m6c" in kernel_feature:
            extra_env["NOS_M6C_ELF"] = str(build_users("m6c", sdk_runtime=use_sdk,
                document_session=use_document_session).resolve())
    if not (ROOT / "Cargo.lock").exists():
        run_logged(["cargo", "+1.90.0", "generate-lockfile"], "lockfile")
    for package, target, feature in (("os-loader", "x86_64-unknown-uefi", loader_feature), ("os-kernel", "x86_64-unknown-none", kernel_feature)):
        args = ["cargo", "+1.90.0", "build", "--locked", "--release", "--target-dir", str(TARGET), "-p", package, "--target", target]
        if feature:
            args += ["--features", feature]
        run_logged(args, f"build-{label}-{package}", extra_env=extra_env if package == "os-kernel" else None)
    loader = TARGET / "x86_64-unknown-uefi/release/os-loader.efi"
    kernel = TARGET / "x86_64-unknown-none/release/os-kernel"
    manifest = {"schema_version": 1, "profile": label, "kernel_feature": kernel_feature,
                "loader_feature": loader_feature, "source_fingerprint_version": 3,
                "source_sha256": source_hash(), "loader": {"sha256": sha256(loader), **inspect_pe(loader)},
                "kernel": {"sha256": sha256(kernel), **inspect_elf(kernel)}}
    manifest["sdk_runtime"] = use_sdk
    manifest["document_session"] = use_document_session
    if qsim_report is not None:
        if kernel_feature in NAS_FEATURES:
            manifest["nas_probe"] = qsim_report
        elif kernel_feature in MATERIAAL_FEATURES:
            manifest["materiaal"] = qsim_report
        elif kernel_feature == "native-display-probe":
            manifest["display_probe"] = qsim_report
        elif kernel_feature in MEMORY_FEATURES:
            manifest["memory"] = qsim_report
        elif kernel_feature in ("native-fpu", "test-fpu-wait"):
            manifest["fpu"] = qsim_report
        elif kernel_feature in ("native-ipc-wait", "native-tpm-crb", "test-tpm-crb-absent"):
            manifest["ipc_wait"] = qsim_report
        elif kernel_feature in ("native-sdk-app", "test-sdk-app-denied"):
            manifest["sdk_app"] = qsim_report
        elif kernel_feature in VOLUME_FEATURES:
            manifest["volume_docs"] = qsim_report
        elif kernel_feature in SUPERVISOR_FEATURES:
            manifest["supervisor"] = qsim_report
        elif kernel_feature in SYSTEM_FEATURES:
            manifest["system"] = qsim_report
            if kernel_feature in ("native-net-service", "test-net-service-client-exit"):
                manifest["network_service"] = {name: qsim_report[f"network_{name}"]
                                               for name in ("service", "client")}
            if kernel_feature in RANDOM_FEATURES:
                manifest["random_probe"] = qsim_report["random_probe"]
            if kernel_feature in IMAGE_FEATURES:
                manifest["image_probe"] = qsim_report["image_probe"]
        elif kernel_feature in KEYRING_FEATURES:
            manifest["keyring"] = qsim_report
        elif kernel_feature in UPDATE_FEATURES:
            manifest["update"] = qsim_report
        else:
            manifest["qsim_user"] = qsim_report
    esp = BUILD / "esp"
    (esp / "EFI/BOOT").mkdir(parents=True, exist_ok=True)
    shutil.copy2(loader, esp / "EFI/BOOT/BOOTX64.EFI")
    if loader_feature == "signed-boot-test":
        # B03: the signed loader only boots from the slot store; leave no
        # unsigned fallback target on the ESP at all.
        (esp / "kernel.elf").unlink(missing_ok=True)
    else:
        shutil.copy2(kernel, esp / "kernel.elf")
    save_json(BUILD / "manifest.json", manifest)
    save_json(BUILD / f"logs/build-{label}-manifest.json", manifest)
    print(f"Built and inspected {label}: {len(manifest['kernel']['segments'])} ELF load segments")
    return manifest


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--loader-feature", choices=["test-ebs-retry", "graphics-g0", "framebuffer", "signed-boot-test"])
    parser.add_argument("--image-version", type=int, help="B03: NOS_IMAGE_VERSION of an update kernel")
    parser.add_argument("--kernel-feature", choices=KERNEL_FEATURES)
    args = parser.parse_args()
    build(args.loader_feature, args.kernel_feature, args.loader_feature or args.kernel_feature or "normal",
          args.image_version)
