# inscribe-flash

Application for uploading keyboard firmware over USB.

## Installing

Download the application from releases.

### Windows

Install the application using the installer (`.msi` file).

> Currently the initial setup on Windows is more complicated than on other platforms
> due to limitations of USB drivers on Windows. We are trying to simplify the setup,
> but for now please follow the instructions below.

On Windows it is necessary to install additional drivers in order to flash firmware over USB.
To do this install [Zadig](https://zadig.akeo.ie/) drivers installer.
Because the keyboard has 2 different modes ("runtime" and "bootloader") driver installation
will have to be done twice.

When you are ready to flash new firmware file:

* Run Zadig, connect your keyboard and click on `Install WCID Driver`.
* Start inscribe-flash, select firmware file and click `Flash`.
* The keyboard will be detached into "bootloader" mode.
* Now run Zadig once again and install drivers with `Install WCID Driver`.
* Close inscribe-flash.

From now on you should be able to flash firmware with inscribe-flash using the regular
procedure. In case this doesn't work on the first attempt, try restarting your system.
If restart doesn't help please submit an issue.

For troubleshooting you can also see [How to use libusb on Windows](https://github.com/libusb/libusb/wiki/Windows#how-to-use-libusb-on-windows).

### Mac

Install the application using the Application Bundle (`.app`) or Disk Image (`.dmg`).

On macOS it is currently necessary to install `libusb`. The simpliest way is to install
[Homebrew](https://brew.sh/) and then run `brew install libusb` from macOS Terminal.
Then you can use inscribe-flash normally.

### Linux

Install from the provided `.deb` package or use the provided AppImage. No additional
dependencies are needed.

To allow access without root permissions (without "sudo") add make sure to save the following
udev rule file at `/etc/udev/rules.d/40-generic-keyboard.rules`:
```
# Allow access to 16c0:27db (generic keyboard) for users in group plugdev
SUBSYSTEMS=="usb", ATTRS{idVendor}=="16c0", ATTRS{idProduct}=="27db", GROUP="plugdev", MODE="0666"
```
and ensure that user is in group `plugdev`. To check it use the command `groups`, if you don't
see `plugdev` then add your user with `sudo usermod -a -G plugdev <your-username>`.

## Usage

Start inscribe-flash then select the firmware file that you want to flash (or drag it
into the area). Connect your keyboard, it should show up under "Detected devices".
Click the "Flash" button. This will detach the keyboard into "Bootloader" mode (if not
already detached) and perform firmware upgrade. When finished, unplug your keyboard and
plug it again. It will now be using the new firmware.

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

This will build only the executables for current system. After building on all systems
generate relases with `utils/release.sh`.
