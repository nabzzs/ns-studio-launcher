# NS Studio Launcher

An experimental Windows launcher and glass-themed UI library for a creative software suite. The launcher presents seven app cards, a grid or table view, light and dark themes, custom accent colors, and a Windows acrylic backdrop where supported.

This repository contains the NS-authored launcher, `ns-glass` theme code, and original icons. It does **not** contain the creative editors, installers, or a ready-to-run suite. The editors used in the local NS Studio prototype are separate forks of [ArtCraft/Storytold projects](https://github.com/storytold); their code and contributions belong to their respective upstream authors.

## Source layout

- `sources/ns-launcher`: Rust/egui launcher prototype
- `sources/ns-glass`: reusable theme and window appearance code
- `branding`: original NS icons used by the launcher

## Build

Install the Rust toolchain on Windows, then run:

```powershell
cd sources/ns-launcher
cargo run
```

The cards expect locally installed editors. Their paths are configured by the launcher and must match your own installation. A solid background is used where the Windows glass effect is unavailable.

## Status and scope

This is an early development prototype. The complete NS Studio suite, editor integration, project import, automatic updates, and production signing are outside this repository. No Adobe file-format or plug-in compatibility is claimed.

## License and attribution

The code in `sources/ns-launcher` and `sources/ns-glass` is MIT licensed; see each directory's `LICENSE`. The original NS icons in `branding` are offered under MIT or Apache-2.0. Third-party Rust dependencies retain their own licenses. The separate ArtCraft/Storytold editors are not part of this repository.
