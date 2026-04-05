#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
QEMU_DIR="${ROOT_DIR}/.qemu"
DISK_PATH="${1:-${QEMU_DIR}/debian-test.qcow2}"
MEMORY_MB="${MEMORY_MB:-4096}"
CPUS="${CPUS:-4}"

if [[ ! -f "${DISK_PATH}" ]]; then
  echo "disk image not found: ${DISK_PATH}" >&2
  exit 1
fi

exec qemu-system-x86_64 \
  -enable-kvm \
  -m "${MEMORY_MB}" \
  -smp "${CPUS}" \
  -cpu host \
  -machine q35,accel=kvm \
  -drive "file=${DISK_PATH},format=qcow2,if=virtio" \
  -device virtio-vga-gl \
  -display gtk,gl=on \
  -device virtio-keyboard-pci \
  -device virtio-mouse-pci \
  -net nic,model=virtio \
  -net user

