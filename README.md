# LifeLog-Logger

This is a logger that you can use to help log your life!

#### Data Modalities

- [x] Screen
- [x] Microphone
- [x] Processes
- [x] Hyprland specific information (like the current window title, all clients)
- [ ] Browser history, browser analytics
- [ ] Application-specific information (like the current song playing on Spotify, current file being used in neovim, etc.)
- [ ] Web-app specific information (instagram messages, youtube watch history (and analytics, videos liked, etc.))
- [ ] Who you are interacting with (like who you are messaging on discord, who you are interacting with in real life)
- [ ] Smartwatch data (like heart rate, steps, etc.)

#### Features

- [ ] Intelligently capture data. I.e. instead of based on time, capture data when events are happening (the screen is changing, the user pressed a button to change windows, etc).

### Benchmarking

Here is an estimate of how much data each modality will generate:

`hyprland` 3600 logs = 6 MB \~= 0.0016666 MB/log
`screen` \~256kb/screenshot

### TODO

Should everything be a database? Should I have one database for each modality? Or should I have one database for all modalities?

### IDEA

Be able to go `back` in time to that event would be very cool.

Todo:

Try fuzz testing, prop testing, hypothesis testing

Clippy
Assertions
Doctests
Examples
Black Box
White Box
Proptest
Fuzzing

cargo
cargo
cargo
cargo
cargo
cargo
cargo
cargo
cargo
run
doc
bench
test
add aws-sdk
install cargo-watch
watch
publish
build --release
H R R R HHHH
run your code 1n debug mode
local package documentation
built-in benchmarking
built-in parallel testing
easily add dependencies
install exes into .cargo/bin
extend cargo and use these
publish packages to crates.io
build release binaries
