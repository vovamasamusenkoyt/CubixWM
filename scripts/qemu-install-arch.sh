#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
QEMU_DIR="${ROOT_DIR}/.qemu"
ISO_PATH="${1:-}"
DISK_PATH="${2:-${QEMU_DIR}/arch-test.qcow2}"
MEMORY_MB="${MEMORY_MB:-4096}"
CPUS="${CPUS:-4}"
SSH_PORT="${SSH_PORT:-2222}"

if [[ -z "${ISO_PATH}" ]]; then
  echo "usage: $0 /path/to/archlinux.iso [disk-path]" >&2
  exit 1
fi

if [[ ! -f "${ISO_PATH}" ]]; then
  echo "iso not found: ${ISO_PATH}" >&2
  exit 1
fi

if [[ ! -f "${DISK_PATH}" ]]; then
  echo "disk image not found: ${DISK_PATH}" >&2
  echo "create it first with scripts/qemu-create-disk.sh ${DISK_PATH}" >&2
  exit 1
fi

mkdir -p "${QEMU_DIR}"

exec qemu-system-x86_64 \
  -enable-kvm \
  -m "${MEMORY_MB}" \
  -smp "${CPUS}" \
  -cpu host \
  -machine q35,accel=kvm \
  -drive "file=${DISK_PATH},format=qcow2,if=virtio" \
  -cdrom "${ISO_PATH}" \
  -boot d \
  -device virtio-vga-gl \
  -display gtk,gl=on \
  -device virtio-keyboard-pci \
  -device virtio-mouse-pci \
  -net nic,model=virtio \
  -net user,hostfwd=tcp:127.0.0.1:${SSH_PORT}-:22
