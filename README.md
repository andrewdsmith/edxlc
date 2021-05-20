# Elite Dangerous X52 Pro LED Control - EDXLC

This small companion app to the game Elite Dangerous automatically controls the
button LEDs on your Saitek X52 Pro joystick so that they reflect the current
state of your ship or SRV in the game.

Currently the app detects if you activate any of the following:

- Landing gear
- Cargo scoop
- External lights
- Frame shift drive
- Silent running

When activated, any button on your X52 Pro (except currently Fire and POV 2)
configured to control the item in question will turn yellow when activated.

Buttons that are cannot be used because of certain other conditions turn red.
For example when mass-locked, all buttons relating to hyperdrive and supercruise
turn red.

Buttons that may require immediate activation alert by flashing red and yellow.
For example when docking the landing gear button flashes and when overheating
the heat sink button flashes.

The app reads the control bindings from the custom bindings file (Odyssey only)
so if you're using any other pre-defined set of bindings it won't work (yet).

The colours used by the app can be configured by editing the `edxlc.toml` file.
This file is created automatically when the app is first run if it does not
exist. You must restart the app to pick up changes in configuration.

Over time more buttons, controls and states will be supported.

The app runs in a console window and can be exited with Ctrl+C.

## Installation

You can run `edxlc.exe` from anywhere but it assumes you have the X52 Pro
drivers installed at the default location of
`C:\Program Files\Logitech\DirectOutput`.