# evdev-accel

evdev based mouse acceleration. Grabs the input device and forwards events via a virtual uinput device.

Largely based on [systemofapwne/leetmouse](https://github.com/systemofapwne/leetmouse).

## Usage

- Add yourself to the `input` usergroup for access to `/dev/input`, and relog.
- Copy [examples/config.toml](examples/config.toml) to `$HOME/.config/evdev-accel/config.toml`
- Edit the config to your liking
- `cargo run`

## TODO

- Add default config if no cfg file
- Add device selection to cfg file for non-interactive mode
- Min/Max accel settings
- Accel offset
- Different accel curves

