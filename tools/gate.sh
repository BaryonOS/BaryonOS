#!/usr/bin/env bash
# SPDX-License-Identifier: MPL-2.0
# tools/gate.sh — Uniforme Kwaliteitspoort voor nieuw-os
set -uo pipefail

echo "🔍 [Gate] Start verificatie voor nieuw-os..."

RUN_CMD="eval"
if command -v rtk &> /dev/null; then
    RUN_CMD="rtk"
fi

# Voer de repo-specifieke checks uit
if ! $RUN_CMD cargo check --lib; then
    echo "❌ Cargo check gefaald." >&2
    exit 2
fi

if ! $RUN_CMD cargo test; then
    echo "❌ Cargo tests gefaald." >&2
    exit 2
fi

if [ -x "tools/qemu-smoke-test.sh" ]; then
    if ! $RUN_CMD ./tools/qemu-smoke-test.sh; then
        echo "❌ QEMU smoke test gefaald." >&2
        exit 2
    fi
elif [ "${GATE_REQUIRE_QEMU:-0}" = "1" ]; then
    echo "❌ QEMU smoke test vereist, maar tools/qemu-smoke-test.sh ontbreekt." >&2
    exit 2
else
    echo "⚠️  QEMU smoke test niet uitgevoerd (tools/qemu-smoke-test.sh ontbreekt)."
fi

echo "✅ Alle uitgevoerde gates geslaagd!"
exit 0
