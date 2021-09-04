# Elite Dangerous X52 Pro Light Control - EDXLC

This small companion app to the game Elite Dangerous automatically controls the
button lights on your Saitek X52 Pro joystick so that they reflect the current
state of your ship or SRV in the game.

The app monitors the state of the following:

- Landing gear
- Cargo scoop
- External lights
- Frame shift drive (FSD)
- Silent running
- Hardpoints
- Night vision

Where a game control for these items is bound to a button on the joystick, the
button light will indicate the following states:

- Inactive - Green (Off) - Not currently activated but can be activated
- Active - Amber (On) - Currently activated
- Blocked - Red (Off) - Cannot be activated
- Alert - Flashing Red/Amber (Flashing) - May need to be activated urgently

An example blocked state is FSD activation while mass-locked. Examples of alert
states include heat sinks when overheating and undeployed landing gear after
docking permission has been granted.

The overall game state also determines how each state is mapped to a colour,
meaning different colours are used when hardpoints are deployed as to normal.

The light behaviour can be configured by editing the `edxlc.toml` file. This
file is created automatically when the app is first run if it does not exist.
You must restart the app to pick up changes in this configuration.

The default configuration is:

```toml
[default]
inactive = ["off", "green"]
active = ["on", "amber"]
blocked = ["off", "red"]
alert = ["flash", "red-amber"]

[hardpoints-deployed]
inactive = ["off", "red"]
active = ["on", "amber"]
blocked = ["off", "off"]
alert = ["flash", "red-amber"]
```

The `hardpoints-deployed` section is optional and will fall back to the values
in `default` if missing.

For each state you specify the light mode for boolean and red/amber/green
lights. For boolean lights, the supported modes are:

- `off`
- `on`
- `flash`

For red/amber/green ligths, the supported modes are:

- `off`
- `green`
- `amber`
- `red`
- `red-amber`

The app reads the control bindings from the custom bindings file (Odyssey only)
so if you're using any other pre-defined set of bindings it won't work (yet).

The app runs in a console window and can be exited with Ctrl+C.

## Installation

You can run `edxlc.exe` from anywhere but it assumes you have the X52 Pro
drivers installed at the default location of
`C:\Program Files\Logitech\DirectOutput`.