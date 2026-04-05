# QEMU test plan for Arch Linux

Use a clean Arch VM without any DE or WM. That keeps the test system close to the final target and avoids debugging around somebody else's compositor.

## Host-side

Create a disk image:

```bash
./scripts/qemu-create-disk.sh
```

Install Arch from the local ISO:

```bash
./scripts/qemu-install-arch.sh /hdd/Загрузки/archlinux-2026.03.01-x86_64.iso
```

Run the installed VM:

```bash
./scripts/qemu-run-arch.sh
```

SSH from the host after enabling `sshd` in the guest:

```bash
ssh -p 2222 vmko@127.0.0.1
```

## Guest-side package baseline

After bootstrapping Arch, install the minimal toolchain and graphics stack:

```bash
sudo pacman -Syu --needed \
  base-devel \
  git \
  rustup \
  pkgconf \
  mesa \
  libglvnd \
  wayland \
  wayland-protocols \
  xorg-xwayland \
  libxkbcommon \
  libinput \
  libdisplay-info \
  seatd \
  vulkan-tools \
  foot \
  weston
rustup default stable
```

## Practical workflow

Recommended development loop:
- host machine: edit code
- guest VM: `git clone` or `git pull`
- guest VM: `cargo run`

That is more reliable than manually copying binaries and keeps dependencies honest.

## Bring-up order

Do not start with bare DRM/KMS.

First target:
- integrate `smithay`
- use a nested backend first
- map `xdg_toplevel`
- focus on click
- move and resize by mouse
- verify no hangs on open/close

Second target:
- Xwayland

Third target:
- DRM/KMS + seat/session work

## Notes

- This repository currently contains the WM core scaffold and QEMU helpers.
- The next implementation step is real Smithay integration inside the existing module layout.
