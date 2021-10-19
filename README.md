# Elite Dangerous X52 Pro Light Control - EDXLC

**EDXLC** is a companion app for the game **Elite Dangerous** that controls the
button lights on your **Saitek X52 Pro** joystick so that they reflect the
current state of your ship in the game.

Download [the latest zip file][download], unzip, and run `edxlc.exe`. The app
runs in a console window. Exit with `Ctrl`+`C` or just close the console
window.

[download]: https://github.com/andrewdsmith/edxlc/releases/download/v1.12/edxlc_v1.12.zip

On first run the app creates a simple text file called `edxlc.toml`. Edit this
file to change how the app behaves. Restart the app after making any changes to
this file. To reset to defaults delete the file.

The app detects the following:

- Landing gear deployed
- Cargo scoop open
- External lights activated
- Frame shift drive (FSD) charging
- Silent running activated
- Hardpoints deployed
- Night vision activated

Where a game control is bound in game to a button on the joystick, the button
light will indicate the following states:

- Inactive - Green (Off) - Not currently activated but can be activated
- Active - Amber (On) - Currently activated
- Blocked - Red (Off) - Cannot be activated
- Alert - Flashing amber (Flashing) - May need to be activated urgently

An example blocked state is FSD charging while mass-locked or landing gear
deployed. Examples of alert states include heat sinks when overheating and
landing gear when docking permission has been granted.

When hardpoints are deployed or night vision is activated the app switches to
an alternative configuration.

The default configurations in `edxlc.toml` are:

```toml
[default]
inactive = ["off", "green"]
active = ["on", "amber"]
blocked = ["off", "red"]
alert = ["flash", "amber-flash"]

[hardpoints-deployed]
inactive = ["off", "red"]
active = ["on", "amber"]
blocked = ["off", "off"]
alert = ["flash", "amber-flash"]

[night-vision]
inactive = ["off", "off"]
active = ["on", "green"]
blocked = ["off", "off"]
alert = ["flash", "green-flash"]
```

The `hardpoints-deployed` and `night-vision` sections are optional and will
fall back to the values in `default` if missing.

For each state you specify the light mode for boolean and red/amber/green
lights. For boolean lights, the supported modes are:

- `off`
- `on`
- `flash`

For red/amber/green ligths, the supported modes are:

- `off`
- `red`
- `amber`
- `green`
- `red-flash`
- `red-amber-flash`
- `red-green-flash`
- `amber-flash`
- `amber-green-flash`
- `amber-red-flash`
- `green-flash`
- `green-amber-flash`
- `green-red-flash`

To use an alternative configuration file specify it as a command line argument:

```
edxlc.exe C:\Path\To\My\config.toml
```

By default the app reads the game control bindings from the `Custom.4.0.binds`
bindings file used by Odyssey. You can use a different bindings file (e.g.
`Custom.3.0.binds` for Horizons) by specifying the full path to the file in the
`bindings` value in the `[files]` section of `config.toml`, e.g.

```toml
[files]
bindings = 'C:\Users\DavidB\AppData\Local\Frontier Developments\Elite Dangerous\Options\Bindings\Custom.3.0.binds'
```

Important: Due to the way the TOML file format works, you should use single
quote characters around the path (as shown above).