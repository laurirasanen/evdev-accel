# evdev-accel

evdev based mouse acceleration. Grabs the input device and forwards events via a virtual uinput device.

Largely based on [systemofapwne/leetmouse](https://github.com/systemofapwne/leetmouse).

## Usage

- Add yourself to the `input` usergroup for access to `/dev/input`, and relog.
- Copy [examples/config.toml](examples/config.toml) to `$HOME/.config/evdev-accel/config.toml`
- Edit the config to your liking
- `cargo install --path .`
- `evdev-accel --help`

See [examples/evdev-accel.desktop](examples/evdev-accel.desktop) for XDG desktop entry.

## TODO

- Add default config if no cfg file
- Retry on device loss
- Min/Max accel settings
- Accel offset
- Different accel curves

