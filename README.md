[![CodeFactor](https://www.codefactor.io/repository/github/klassenserver7b/danceinterpreter-rs/badge)](https://www.codefactor.io/repository/github/klassenserver7b/danceinterpreter-rs)
[![CI](https://github.com/klassenserver7b/danceinterpreter-rs/actions/workflows/check_and_test.yml/badge.svg)](https://github.com/klassenserver7b/danceinterpreter-rs/actions/workflows/cargo-check.yml)

# danceinterpreter-rs

The Danceinterpreter is a utility to display songs and their according dances at ballroom dance events

It currently contains three source modes

1. M3U / M3U8 files with references to local mp3 files
2. A connection to a running Traktor Pro instance
3. Manually added songs and static dance labels

## Roadmap

The [Roadmap](https://github.com/users/klassenserver7b/projects/1) can be found here

# Installation

The danceinterpreter is currently only availible for Linux(x64, aarch64) and Windows(x64).

_macOS support is technically possible but as long as it is not requested I don't want to waste actions minuites on
those builds._
_Any other OS/Arch could be possible if rust and my used libraries support it_

The **[Flathub
](https://github.com/klassenserver7b/danceinterpreter-rs/tree/flatpak-packaging?tab=readme-ov-file#flathub---linux-only---preferred)**
release (**Linux x64**) is the **ONLY** one which is actively tested.

## Flathub - Linux only - preferred

### Installation

[![https://flathub.org/apps/io.github.klassenserver7b.danceinterpreter-rs](https://flathub.org/api/badge?locale=en)](https://flathub.org/apps/io.github.klassenserver7b.danceinterpreter-rs)

```BASH
flatpak install flathub io.github.klassenserver7b.danceinterpreter-rs
```

### Update

Update via your distros software store or run

```BASH
flatpak update io.github.klassenserver7b.danceinterpreter-rs
```

### Uninstall

Uninstall via your distros software store or run

```BASH
flatpak uninstall io.github.klassenserver7b.danceinterpreter-rs
```

## Cargo - all platforms

### Installation

1. Make sure you have cargo and the rust stack installed, if
   not [get started here](https://www.rust-lang.org/learn/get-started)
2. Run `cargo install danceinterpreter-rs`
3. Start the danceinterpreter from you console by running `danceinterpreter-rs`

### Update

Rerun `cargo install danceinterpreter-rs`

### Uninstall

Run `cargo uninstall danceinterpreter-rs`

# Support

Always feel free to open an issue according to the issue templates at this GitHub page.

# Troubleshooting

## Drag & drop does nothing on Wayland

If dropping audio files onto the window does nothing (no reaction, no
console output), you are running the app as a native Wayland window — the
winit windowing library used here does not implement file drops on Wayland,
so no events ever reach the app. Running through XWayland works. Two options:

- Start with `env -u WAYLAND_DISPLAY cargo run` (or unset `WAYLAND_DISPLAY`
  in your launcher) to force the X11 backend via XWayland.
- Use the **Add audio files** button in the playlist toolbar; the file
  dialog works in every environment.

# Contributing
- see [CONTRIBUTING.md](CONTRIBUTING.md)

## Screenshots

![MainWindow](https://github.com/user-attachments/assets/896e5cf3-832b-4579-aba5-398d5075aeee)
![DanceWindow](https://github.com/user-attachments/assets/f7d7c43e-31cb-4c9e-a593-1a664d9190b4)
