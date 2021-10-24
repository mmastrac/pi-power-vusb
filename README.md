# pi-power-vusb

This project gives the ability for you to run a USB-powered GPIO board on the dirt-cheap-BOM DigiSpark and
clone boards.

The on-device code is based on a DigiSpark sample and the V-USB code, both licenced under GPLv2/v3. The additional
code for EEPROM management and startup was written by the author of this repository and is under the same license.

The Rust interface code is entirely written by the author of this repository, and is GPLv3-only.

## Usage

Incomplete, coming soon.

## TODO

- [ ] Create a docker-based build to build the AVR code
- [ ] Build releases for Linux (x32/x64), OSX (Intel/ARM), Windows (if possible)
- [ ] Allow programming a serial number into the EEPROM

