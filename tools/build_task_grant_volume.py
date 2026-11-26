"""A03-T2c-2: bind the task-grant broker, its client and the unchanged volume
service ELF to the native-system slots of `kernel/src/m2/m3/task_grant_volume.rs`.

Builds only; never boots or grants authority. The volume ELF comes from the
existing NAS leaf builder (volume-docs service, volume-docs linker script).
"""
from __future__ import annotations

import sys
from pathlib import Path

from build_nas import build_users as build_nas_users
from build_users import inspect_user
from common import ROOT, TARGET, run_logged, sha256


def build_users():
    nas_env, _ = build_nas_users()
    volume = Path(nas_env["NOS_VOLUME_SERVICE_ELF"])
    target = TARGET / "sdk-task-grant"
    run_logged([sys.executable, "-B", str(ROOT / "sdk/examples/task-grant-broker/build.py")],
               "build-task-grant-example", extra_env={"CARGO_TARGET_DIR": str(target)})
    client = target / "x86_64-unknown-none/release/task-grant-client"
    broker = target / "x86_64-unknown-none/release/task-grant-broker"
    report = {
        "client_sha256": sha256(client), "client_elf": inspect_user(client),
        "broker_sha256": sha256(broker), "broker_elf": inspect_user(broker),
        "volume_sha256": sha256(volume), "volume_elf": inspect_user(volume),
    }
    # Slots 0 broker, 1 owner, 2 volume, 3 helper (m3.rs process_elf order).
    return {
        "NOS_SYSTEM_INIT_ELF": str(broker.resolve()),
        "NOS_KEYRING_SERVICE_ELF": str(client.resolve()),
        "NOS_VOLUME_SERVICE_ELF": str(volume.resolve()),
        "NOS_SYSTEM_APP_ELF": str(client.resolve()),
    }, report
