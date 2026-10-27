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

### Folder access in the Flatpak sandbox

The Flatpak build is sandboxed. Loading a playlist works like this:

1. **Pick a folder** — the folder you pick in the *Open playlist* dialog is
   granted to the app (including all subfolders).
2. **Playlists are discovered inside it** — every `.m3u`/`.m3u8` in the
   granted folder tree is found. A single playlist loads automatically,
   several ones offer a small chooser dialog.
3. **All paths stay inside the grant** — playlist entries (relative paths,
   percent-encoded names and `file://` URIs) resolve against the playlist's
   location inside the granted folder, so the audio files are readable even
   though the rest of your home stays off limits.

Notes:

- Entries pointing *outside* the granted folder cannot be read by a
  sandboxed app. The toast after loading tells you how many tracks were
  skipped (`Loaded — 12/40 tracks, 28 not accessible`), and the skipped
  entries are printed to the console.
- The Flathub build grants read-only access to `~/Music`, `/media` and
  `/run/media` (removable drives). Drag & drop and opening files from those
  locations work without picking a folder first; from other places (e.g.
  `~/Dokumente`) drag & drop cannot work in the sandbox — use the *Open
  playlist* folder flow instead.
- You can grant additional locations yourself with
  [Flatseal](https://flathub.org/apps/com.github.tchx84.Flatseal)
  (*Filesystem → Other files*, e.g. `~/Dokumente`). If a playlist lives
  somewhere unusual and drag & drop fails with a sandbox hint, that is the
  way to fix it.

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
console output), you are running the app as a native Wayland window. The
winit windowing library used here does not implement file drops on Wayland
at all — no winit release does (still open upstream in
[winit#720](https://github.com/rust-windowing/winit/issues/720) and
[winit#4773](https://github.com/rust-windowing/winit/issues/4773)), so no
events ever reach the app. Running through XWayland works. Two options:

- Start with `env -u WAYLAND_DISPLAY cargo run` (or unset `WAYLAND_DISPLAY`
  in your launcher) to force the X11 backend via XWayland.
- Use the **Add audio files** button in the playlist toolbar; the file
  dialog works in every environment.

# Contributing
- see [CONTRIBUTING.md](CONTRIBUTING.md)

## Screenshots

![MainWindow](https://github.com/user-attachments/assets/896e5cf3-832b-4579-aba5-398d5075aeee)
![DanceWindow](https://github.com/user-attachments/assets/f7d7c43e-31cb-4c9e-a593-1a664d9190b4)
