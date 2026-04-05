#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
QEMU_DIR="${ROOT_DIR}/.qemu"
DISK_PATH="${1:-${QEMU_DIR}/arch-test.qcow2}"
DISK_SIZE="${2:-24G}"

mkdir -p "${QEMU_DIR}"
qemu-img create -f qcow2 "${DISK_PATH}" "${DISK_SIZE}"
echo "created disk: ${DISK_PATH}"
