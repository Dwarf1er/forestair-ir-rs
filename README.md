<div align="center">

# forest-air-ir-rs
#### Rust implementation of the ForestAir AC IR protocol

<img alt="idfmgr logo" src="https://raw.githubusercontent.com/Dwarf1er/forest-air-ir-rs/master/assets/forestair-ir-rs-logo.png" height="250" />

![License](https://img.shields.io/github/license/Dwarf1er/forestair-ir-rs?style=for-the-badge)
![Issues](https://img.shields.io/github/issues/Dwarf1er/forestair-ir-rs?style=for-the-badge)
![PRs](https://img.shields.io/github/issues-pr/Dwarf1er/forestair-ir-rs?style=for-the-badge)
![Contributors](https://img.shields.io/github/contributors/Dwarf1er/forestair-ir-rs?style=for-the-badge)
![Stars](https://img.shields.io/github/stars/Dwarf1er/forestair-ir-rs?style=for-the-badge)

</div>

## Project Description

This project implements an infrared transmitter for controlling [ForestAir](https://forestair.ca/) air conditioners. The transmitter is built using an ESP32 with an IR transmitter and the RMT (remote control) peripheral. It has been built after reverse-engineering an original ForestAir IR remote and reproducing its 35-bit pulse-distance encoded protocol.

## Table of Contents

<!-- mtoc-start -->

* [Future Plans](#future-plans)
* [Hardware Requirements](#hardware-requirements)
* [Project Template](#project-template)
* [System prerequisites](#system-prerequisites)
* [Setup Script](#setup-script)
* [ESP-IDF Submodules](#esp-idf-submodules)
* [User Permissions for Serial](#user-permissions-for-serial)
* [Building and Flashing](#building-and-flashing)
* [ForestAir IR Protocol Overview](#forestair-ir-protocol-overview)
  * [IR Timing Specification](#ir-timing-specification)
  * [Pulse-Distance Encoding Diagram](#pulse-distance-encoding-diagram)
  * [Frame Format](#frame-format)
  * [Fixed Header Constant](#fixed-header-constant)
  * [Payload Format](#payload-format)
    * [AC Modes Encoding](#ac-modes-encoding)
    * [Fan Speeds Encoding](#fan-speeds-encoding)
    * [Tempearture Encoding](#tempearture-encoding)
    * [Transmission Order](#transmission-order)
  * [Example Payload and Encoding](#example-payload-and-encoding)
* [Troubleshooting](#troubleshooting)
* [License](#license)

<!-- mtoc-end -->

## Future Plans

This project is currently a **proof of concept** to demonstrate the basics of the ForestAir IR protocol on an ESP32. In the future, I plan to expand the functionality to include:
* Web interface to allow users on the same network as the ESP32 to access a page through mDNS (e.g., `ac.local`)
* API for automation to allow integrations with other platforms such as `Home Assistant` or `ESPHome`

## Hardware Requirements

* ESP32 (any variant with RMT peripheral)
* IR Transmitter (IR LED + driver)

Default RMT setup:
```bash
TX Pin: GPIO14
RMT Channel: 0
Carrier: 38 kHz @ 33% duty
```

## Project Template

This project was generated using [the esp-idf-template](https://github.com/esp-rs/esp-idf-template)
```bash
cargo install cargo-generate
cargo generate esp-rs/esp-idf-template cargo
```

## System prerequisites

To get started with this project, make sure to install the following dependencies on your system.

Arch based (via Pacman):
```bash
sudo pacman -S git cmake ninja python python-pip python-virtualenv dfu-util libusb ccache gcc pkg-config clang llvm libxml2 libxml2-legacy dotenv
```

Debian based (via apt):
```bash
sudo apt-get install git wget flex bison gperf python3 python3-pip python3-venv cmake ninja-build ccache libffi-dev libssl-dev dfu-util libusb-1.0-0 dotenv
```

## Setup Script

After having installed the dependencies and having cloned the repository, run the setup script at the root of the repository:
```bash
chmod +x ./setup.sh
./setup.sh
```

## ESP-IDF Submodules

ESP-IDF uses git submodules for its dependencies. You need to initialize them:
```bash
cd .embuild/espressif/esp-idf/v<version number>
git submodule update --init --recursive
```

## User Permissions for Serial

On Arch based distros, the `uucp` group has access to /dev/ttyUSB* devices (similar to `dialout` on Debian based distros). Add your user to this group and run `newgrp` to active your new permissions:
```bash
sudo usermod -aG uucp $USER
newgrp uucp
```

## Building and Flashing

Run the following command to build the project:
```
cargo build
```

Once the project has been built, you can flash it to your ESP32 using the `espflash` tool:
```bash
# command format: espflash flash target/<mcu-target>/debug/<your-project-name>
# see https://github.com/esp-rs/esp-idf-template?tab=readme-ov-file#flash for more details
# example:
espflash flash target/xtensa-esp32-espidf/debug/forestair-ir-rs --monitor
```

## ForestAir IR Protocol Overview

The ForestAir AC units use a 35-bit IR protocol with the following spec:
* 38 kHz carrier frequency
* Pulse-distance encoding with LSB-first transmission
* Fixed header and parametrized payload
* Transmission with no repeat frames (one transmission of the full state after each button press)

### IR Timing Specification

|     Parameter     |          Value         |
| :---------------: | :--------------------: |
| Carrier frequency |         38 kHz         |
|     Duty cycle    |           33%          |
|      Bit mark     |         650 µs         |
|  Logic "0" space  |         550 µs         |
|  Logic "1" space  |         1650 µs        |
|      Stop bit     | 650 µs mark + no space |

### Pulse-Distance Encoding Diagram

```plaintext
Bit '0'  ───650µs HIGH───550µs LOW──────────
Bit '1'  ───650µs HIGH──────────1650µs LOW──
```

### Frame Format

A full IR frame will consist of the following:
* Header mark + header space
* 35 data bits (LSB first)
* Stop bit (650 µs high)

### Fixed Header Constant

0x250000000

### Payload Format

|  Bits |    Field    | Size | Description               |
| :---: | :---------: | ---- | ------------------------- |
|  0–2  |    acMode   | 3    | Operating mode            |
|   3   |    onOff    | 1    | Power state               |
|  4–6  |   fanMode   | 3    | Fan speed                 |
|   7   |    swing    | 1    | Swing on/off              |
|  8–11 | temperature | 4    | 16–30 °C encoded as index |
| 12–15 |   reserved  | 4    | Always 0                  |

#### AC Modes Encoding

| Value |     Mode    |
| :---: | :---------: |
|   0   |     Auto    |
|   1   |     Cool    |
|   2   |  Dehumidify |
|   3   | Ventilation |
|   4   |     Heat    |


#### Fan Speeds Encoding

| Value |  Speed |
| :---: | :----: |
|   0   |  Auto  |
|   1   |   Low  |
|   2   | Medium |
|   3   |  High  |

#### Tempearture Encoding

| Value |  Temperature |
| :---: | :----------: |
|   0   |     16°C     |
|   1   |     17°C     |
| [...] |     [...]    |
|  14   |     30°C     |


#### Transmission Order

Least-significant-bit first (LSB first)

### Example Payload and Encoding

Example settings:
```plaintext
Mode:        Ventilation (3)
Power:       On (1)
Fan:         Low (1)
Swing:       Off (0)
Temperature: 24°C → index 8
```

Payload fields:
```plaintext
acMode      = 3     (bits 0–2)
onOff       = 1     (bit 3)
fanMode     = 1     (bits 4–6)
swing       = 0     (bit 7)
temperature = 8     (bits 8–11)
reserved    = 0     (bits 12–15)
```

Final frame:
```plaintext
rawFrame = 0x250000000 | payload
```

## Troubleshooting

My ForestAir AC unit doesn't seem to turn on if the "Heat" mode is selected, whether on the original remote or with my ESP32. Therefore, if your AC doesn't turn on:
* Try selecting: `Ventilation`, `Cool`, or `Auto` instead
* Confirm temperature index is between 16-30

## License

This software is licensed under the [MIT license](LICENSE)
