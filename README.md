# cubixWM

Minimal Rust window manager/compositor workspace layout.

Current state:
- compiles
- has a real module structure
- contains a small WM core with window state, focus, stacking, move and resize
- uses stub backends/protocols/rendering so the architecture can grow cleanly

Next step after this scaffold:
- integrate `smithay`
- bring up a `winit` backend
- wire `xdg_shell`, `seat`, outputs and rendering
- then add `Xwayland`

Run:

```bash
cargo run -p cubixwm -- run
```

Run the built-in smoke demo:

```bash
cargo run -p cubixwm -- demo
```

QEMU helpers for a clean Arch Linux VM live in `scripts/` and `docs/qemu-arch.md`.
