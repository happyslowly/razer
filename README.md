# razer

A small Linux CLI for querying and configuring supported Razer devices over
HID. This is a learning project based on the protocol documented by
[OpenRazer](https://github.com/openrazer/openrazer).

## Supported devices

- Razer Basilisk V3 Pro wireless (`1532:00ab`)

The device table is intentionally conservative: a device is only opened when
its product ID, interface number, usage page, and usage match a known profile.

## Features

- Read battery level and charging status
- Read firmware version
- Read the current X and Y DPI
- Set equal or independent X and Y DPI values

## Usage

```console
$ razer battery
Razer Basilisk V3 Pro
Battery: 73%

$ razer firmware
Razer Basilisk V3 Pro
Version: 2.50

$ razer dpi
Razer Basilisk V3 Pro
DPI: X = 1600, Y = 1600
```

Set both axes to the same DPI:

```console
$ razer dpi 1600
```

Set the axes independently:

```console
$ razer dpi 1600 800
```

Accepted DPI values currently range from `100` to `30000`.

## Installation with Nix

Run directly from the repository:

```console
$ nix run github:happyslowly/razer -- battery
```

Install into the current Nix profile:

```console
$ nix profile install github:happyslowly/razer
```

For local development:

```console
$ nix develop
$ cargo run -- battery
```

The traditional Nix interface is also available:

```console
$ nix-build
$ ./result/bin/razer battery
```

## Building with Cargo

The `hidapi` Linux native backend requires `pkg-config` and the libudev
development files. After installing those packages for your distribution:

```console
$ cargo build --release
$ ./target/release/razer battery
```

## Device permissions

The current user must have read and write access to the matching `/dev/hidraw*`
device. If the program reports a permission error, configure an appropriate
udev rule for vendor ID `1532` and your system's user-access policy. Avoid
running the program as root as a permanent solution.

## Protocol notes

Razer feature reports use a 90-byte protocol frame wrapped in a HID feature
report. This project validates the report ID, status, transaction ID, command,
data size, and XOR checksum before interpreting response arguments.

The currently implemented protocol calls are:

| Operation | Class | Command ID | Data size |
| --- | ---: | ---: | ---: |
| Battery level | `0x07` | `0x80` | 2 |
| Charging status | `0x07` | `0x84` | 2 |
| Firmware information | `0x00` | `0x81` | 2 |
| Get DPI | `0x04` | `0x85` | 7 |
| Set DPI | `0x04` | `0x05` | 7 |

The Basilisk V3 Pro uses transaction ID `0x1f` for these calls. Set DPI stores
X and Y as big-endian 16-bit values and recalculates the report checksum after
writing the request arguments.

## Acknowledgements

Protocol behavior was cross-checked against the
[OpenRazer mouse driver](https://github.com/openrazer/openrazer/blob/master/driver/razermouse_driver.c)
and
[common Razer protocol implementation](https://github.com/openrazer/openrazer/blob/master/driver/razerchromacommon.c).
