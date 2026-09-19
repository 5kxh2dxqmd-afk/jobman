# Jobman

> [!NOTE]
> A Simple Kindle Jailbreak Manager, that Performs Certain Jobs! :) WIP

<a href='https://ko-fi.com/W7W31J9IS0' target='_blank'><img height='36' style='border:0px;height:36px;' src='https://storage.ko-fi.com/cdn/kofi5.png?v=6' border='0' alt='Buy Me a Coffee at ko-fi.com' /></a>

*Like [my](https://penguins184.xyz/) work? Consider donating or just starring my repo! :)*

<img width="350" height="500" alt="image" src="https://github.com/user-attachments/assets/1de9a1cb-c071-4f6b-aee1-af48b05b0da2" />


## Features

- Current:
    - Update blocking/unblocking
    - Battery health estimates
    - Environment updates
    - Self-updater
    - USB SSH (`g_ether`)
- Planned:
    - Wi-Fi SSH (`iptables`)
    - Update sideloading (akin to Magisk sideload to inactive slot)
    - UI for mounting root overlays on reboot (rootless)

## Building

Great thanks to [slint-kindle-backend](https://github.com/sverrejb/slint-kindle-backend).

### Prerequisites

- [koxtoolchain & Kindle SDK](https://kindlemodding.org/kindle-dev/gtk-tutorial/prerequisites.html)
- cargo-zigbuild
    - `rustup target add armv7-unknown-linux-musleabihf`
    - `cargo install cargo-zigbuild`
    - `sudo pacman -S zig # or platform equivalent`

## PC

Use `slint-viewer`, bundled with the Slint vscode plugin, or:

`./build.sh`

## Kindle

Run `./build.sh kindle`, and copy both folders in `build/` to the device.

## Notes

Inter, Libre Baskerville uses **SIL Open Font License (OFL) Version 1.1**. This project is licensed under **GNU GPLv3**.
