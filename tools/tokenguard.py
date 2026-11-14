#!/usr/bin/env python3
# SPDX-License-Identifier: MPL-2.0
"""Run the shared plugin helper without maintaining a second implementation."""
from pathlib import Path
import runpy


if __name__ == "__main__":
    runpy.run_path(
        str(Path(__file__).resolve().parents[1] / "plugins" / "token-guard"
            / "skills" / "token-guard" / "scripts" / "tokenguard.py"),
        run_name="__main__",
    )
