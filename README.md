# inscribe-flash

Application for uploading keyboard firmware over USB.

## Installation

Download the application from [releases](https://github.com/inscribio/inscribe-flash/releases/latest)
selecting correct version for your device:

* Windows: `*_x64_en-US.msi`
* MacOs x86_64: `*_x64.app.zip` or `*_x64.dmg`
* MacOs ARM64: `*_aarch64.app.zip` or `*_aarch64.dmg`
* Linux: `*_amd64.AppImage` or `*_amd64.deb`

### Windows

On Windows it might be necessary to install additional USB drivers.
When you run the application it will detect if drivers are missing and you will
be prompted to automatically install them.
Under the hood it will use [winusb-installer based on libwdi](https://github.com/inscribio/winusb-installer).

### Mac

There should be no need for any additional configuration as all the required
dependencies are included in the application.

### Linux

You will need libusb installed on your system (it is installed by default on most distributions).

To allow access without root permissions (without "sudo") make sure to save the following
udev rule file at `/etc/udev/rules.d/40-generic-keyboard.rules`:
```
# Allow access to 16c0:27db (generic keyboard) for users in group plugdev
SUBSYSTEMS=="usb", ATTRS{idVendor}=="16c0", ATTRS{idProduct}=="27db", GROUP="plugdev", MODE="0666"
```
and ensure that user is in group `plugdev`. To check it use the command `groups`, if you don't
see `plugdev` then add your user with `sudo usermod -a -G plugdev <your-username>`.

## Usage

Start `inscribe-flash` then select the firmware file that you want to flash (or drag it
into the area). Connect your keyboard, it should show up under "Detected devices".
Optionally select the device to be used by clicking on it (or use the one selected by default).
Click the "Flash" button. This will detach the keyboard into "Bootloader" mode (if not
already detached) and perform firmware upgrade. When finished, unplug your keyboard and
plug it again (might not be needed depending on the system).
The keyboard will now be using the new firmware.

## Development

Tauri application consists of Vue (vue-cli) frontend and Rust backend.
Tauri dependencies (see their website) and Node.js need to be installed.

First install project dependencies:
```sh
npm install .
```

To start development server:

```sh
npm run tauri:serve
```

To build executable for release:

```sh
npm run tauri:build
```
