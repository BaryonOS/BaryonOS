#!/usr/bin/env python3
"""Manual mutation check for src/transport_model.rs (TASK-H03-TRANSPORT-MODEL-1).

Each mutant applies one exact, unique text delta, runs the locked test suite,
records the exit status and restores the original bytes (sha256-verified).
A mutant is "caught" when `cargo test` exits non-zero. Run from
drivers/virtio-snd with the pinned toolchain in PATH.
"""
import hashlib
import pathlib
import subprocess
import sys

SRC = pathlib.Path("src/transport_model.rs")
CMD = ["cargo", "+1.90.0", "test", "--offline", "--locked", "-q", "transport_model"]

MUTANTS = [
    ("M1", "I1 fence accepts status 0 without BME-off read-back",
     "if !(self.status_zero && self.bme_off) {", "if !self.status_zero {"),
    ("M2", "I2 lease/epoch check removed",
     "if lease == self.lease {", "if true {"),
    ("M3", "I3 fence timeout goes Offline instead of Quarantined",
     "now >= self.deadline) {\n            self.state = State::Quarantined;",
     "now >= self.deadline) {\n            self.state = State::Offline;"),
    ("M4", "I4 submit after revoke allowed (revoked check removed)",
     "        if self.revoked {\n            return Err(Reject::Revoked);\n        }\n        self.live()?;",
     "        self.live()?;"),
    ("M5", "I5 rx buffer allowed",
     "Err(Reject::RxForbidden)", "Ok(())"),
    ("M6", "I6 double completion accepted (any slot counts as published)",
     "== Some(Slot::Published);", "!= None;"),
    ("M7", "I6/F19 double release accepted",
     "if self.released {", "if false {"),
]


def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def main() -> int:
    original = SRC.read_bytes()
    base = sha(original)
    print(f"source {SRC} sha256 {base}")
    print(f"command {' '.join(CMD)}")
    caught = 0
    try:
        for mid, what, old, new in MUTANTS:
            text = original.decode()
            if text.count(old) != 1:
                print(f"{mid} ERROR delta not unique/present")
                return 2
            SRC.write_text(text.replace(old, new))
            result = subprocess.run(CMD, capture_output=True, text=True)
            failed = [l for l in result.stdout.splitlines() if l.startswith("    transport_model::tests::")]
            verdict = "CAUGHT" if result.returncode != 0 else "SURVIVED"
            caught += result.returncode != 0
            print(f"{mid} {verdict} exit={result.returncode} {what}")
            print(f"  delta: {old!r} -> {new!r}")
            for line in failed:
                print(f"  red: {line.strip()}")
            SRC.write_bytes(original)
    finally:
        SRC.write_bytes(original)
    restored = sha(SRC.read_bytes())
    print(f"restored sha256 {restored} {'OK' if restored == base else 'MISMATCH'}")
    print(f"caught {caught}/{len(MUTANTS)}")
    return 0 if caught == len(MUTANTS) and restored == base else 1


if __name__ == "__main__":
    sys.exit(main())
