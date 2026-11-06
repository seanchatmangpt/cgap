#!/usr/bin/env bash
# court_cross_platform_qualification.sh
# Verifies cross-compilation qualification across non-mac platforms:
# 1. Linux x86_64 (Xeon / EPYC)
# 2. Linux aarch64 (AWS Graviton / Ampere)
# 3. Bare-metal RISC-V (riscv32imc XIP sovereign microcontroller)
set -euo pipefail

WORKSPACE_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
CGAP_ROOT="${WORKSPACE_ROOT}/cgap"
ARCUS_ROOT="${WORKSPACE_ROOT}/arcus"

echo "================================================================================"
echo "STARTING CROSS-PLATFORM HARDENING QUALIFICATION COURT"
echo "Targets: Linux x86_64, Linux aarch64, Bare-Metal riscv32imc"
echo "================================================================================"

echo "[1/5] Qualifying cgap on Linux x86_64 (x86_64-unknown-linux-gnu)..."
cd "${CGAP_ROOT}"
cargo check --target x86_64-unknown-linux-gnu --bin cgap_e2e_consumer
echo "  [OK]: Linux x86_64 cgap qualification clean."

echo "[2/5] Qualifying cgap on Linux aarch64 (aarch64-unknown-linux-gnu)..."
cd "${CGAP_ROOT}"
cargo check --target aarch64-unknown-linux-gnu --bin cgap_e2e_consumer
echo "  [OK]: Linux aarch64 cgap qualification clean."

echo "[3/5] Qualifying cgap-arrow on Bare-Metal RISC-V (riscv32imc-unknown-none-elf)..."
cd "${CGAP_ROOT}"
cargo check --target riscv32imc-unknown-none-elf --no-default-features -p cgap-arrow
echo "  [OK]: Bare-metal RISC-V cgap-arrow qualification clean."

echo "[4/5] Qualifying arcus producer on Linux amd64 (GOOS=linux GOARCH=amd64)..."
cd "${ARCUS_ROOT}"
GOOS=linux GOARCH=amd64 go build -o /dev/null cmd/arcus_e2e_producer/main.go
echo "  [OK]: Linux amd64 arcus producer qualification clean."

echo "[5/5] Qualifying arcus producer on Linux arm64 (GOOS=linux GOARCH=arm64)..."
cd "${ARCUS_ROOT}"
GOOS=linux GOARCH=arm64 go build -o /dev/null cmd/arcus_e2e_producer/main.go
echo "  [OK]: Linux arm64 arcus producer qualification clean."

echo "================================================================================"
echo "CROSS-PLATFORM QUALIFICATION STATUS: 5/5 TARGETS ADMITTED & CONFORMANT"
echo "================================================================================"
