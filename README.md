# evdev-accel

evdev based mouse acceleration. Grabs the input device and forwards events via a virtual uinput device.

Largely based on [systemofapwne/leetmouse](https://github.com/systemofapwne/leetmouse).

## Usage

- Add yourself to the `input` usergroup for access to `/dev/input`, and relog.
- Copy [examples/config.toml](examples/config.toml) to `$HOME/.config/evdev-accel/config.toml`
- Edit the config to your liking
- `cargo install --path .`
- `evdev-accel --help`

See [examples/evdev-accel-service.sh](examples/evdev-accel-service.sh) for auto-retry on device loss.

## TODO

- XDG desktop entry -- I couldn't get one to work on login, only with manual exec after login
- Add default config if no cfg file
- Min/Max accel settings
- Accel offset
- Different accel curves

