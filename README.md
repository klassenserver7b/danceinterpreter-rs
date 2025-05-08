[![CodeFactor](https://www.codefactor.io/repository/github/klassenserver7b/danceinterpreter-rs/badge)](https://www.codefactor.io/repository/github/klassenserver7b/danceinterpreter-rs)
# danceinterpreter-rs

The Danceinterpreter is a utility to display songs and their according dances at ballroom dance events

It currently contains three source modes
1. M3U / M3U8 files with references to local mp3 files
2. A connection to a running Traktor Pro instance
3. Manually added songs and static dance labels

# Installation

The danceinterpreter is currently availible for all platforms which are supported by the rust language
Altough only the [**Flathub**](https://github.com/klassenserver7b/danceinterpreter-rs/tree/flatpak-packaging?tab=readme-ov-file#flathub---linux-only---preferred) release is actively tested

## Flathub - Linux only - preferred
### Installation
![https://flathub.org/apps/io.github.klassenserver7b.danceinterpreter-rs](https://flathub.org/api/badge?locale=en)

### Update
Update via your Software Store or run `flatpak update io.github.klassenserver7b.danceinterpreter-rs`

### Uninstall
Uninstall via your Software Store or run `flatpak uninstall io.github.klassenserver7b.danceinterpreter-rs`

## Cargo - all platforms
### Installation
1. Make sure you have cargo and the rust stack installed, if not [get started here](https://www.rust-lang.org/learn/get-started)
2. Run `cargo install danceinterpreter-rs`
3. Start the danceinterpreter from you console by running `danceinterpreter-rs`

### Update
Rerun `cargo install danceinterpreter-rs`

### Uninstall
Run `cargo uninstall danceinterpreter-rs`

# Support
Always feel free to open an issue according to the issue templates at this github page.


## Screenshots
![MainWindow](https://github.com/user-attachments/assets/5d378bd7-44f2-418a-bd0b-99e5add67c6d)
![DanceWindow](https://github.com/user-attachments/assets/a9775faa-6701-43b0-8532-3e400a4a0592)
