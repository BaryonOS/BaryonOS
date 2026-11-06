#!/usr/bin/env bash
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
fi

echo "✅ Alle gates geslaagd!"
exit 0
