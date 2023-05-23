# inscribe-flash

Application for uploading keyboard firmware over USB.

![inscribe-flash](https://github.com/inscribio/inscribe-flash/assets/16623787/04da3aaf-6645-4ac0-bd0d-1bcbb375d16d)

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

## Troubleshooting

If you cannot detach the device (to bootloader) make sure to press the AllowBootloader button -
by default keyboard will not allow detaching before the button is pressed. You can find the location
of this button for your firmware in the online configurator.

If you don't know which button has the AllowBootloader function, you will have to manually detach the
keyboard to bootloader. To do so:

* Connect the keyboard to PC via USB cable. Then, on the half connected via USB:
* Find a small hole in the upper part of keyboard case, located slightly above the thumb cluster.
* Use a toothpick or similar object to push a button and hold it for 3 seconds.
* The keyboard should detach to bootloader and you will see this in inscribe-flash

### Windows

USB drivers on Windows are more problematic than on Mac/Linux. In order to detach/flash the keyboard
WinUSB driver must be loaded in the system. The keyboard in its "main mode" uses Microsoft OS 2.0 descriptors
to instruct the system to automatically load WinUSB driver for the device. This way detaching to bootloader
should work out of the box. The bootloader requires installation of WinUSB driver, which is automatically done
by inscribe-flash application.

If detaching or flashing does not work, try to uninstall all drivers for the keyboard/bootloader by following
these steps:

* Delete all registry entries for this device
    * This is needed because Windows reads special MS OS 2.0 descriptors only when the device is connected for the first time
    * Open Registry Editor (type it in application menu or use Run -> "regedit")
    * Go to `HKEY_LOCAL_MACHINE\SYSTEM\CurrentControlSet\Control\usbflags`
    * Find all entries starting with `16C027DB` (e.g. `16C027DB0001`), which corresponds to keyboard's VID:PID which is 16C0:27DB
    * Click on each entry and select "Delete"
* Uninstall USB drivers for the keyboard and bootloader
    * Open Device Manager (right click on start menu)
    * Find device "ghanima keyboard" ("main mode" of operation of the keyboard)
    * Right-click -> Uninstall device -> Check the "Delete driver..." checkbox -> Confirm
    * Now detach the keyboard to bootloader manually as described at the beginning of Troubleshooting section
    * Find device called "STM32 BOOTLOADER" and uninstall drivers the same as before
