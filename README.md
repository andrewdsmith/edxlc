# Elite Dangerous X52 Pro LED Control - EDXLC

The aim of this small companion app is to automatically control the button LEDs
on your Saitek X52 Pro joystick using the state of your ship or SRV in the game
Elite Dangerous.

Currently the app toggles the LEDs on the T2, T4 or T6 buttons (only) if bound
to:

- Landing gear
- Cargo scoop
- External lights

The app reads the control bindings from the custom bindings file (Horizons not
Odyssey) so if you're using any other pre-defined set of bindings it won't work.

The intent is that over time more buttons, controls and states will be
supported. Additionally, the app will become configurable to set the desired
colours for conditions such as active, in use and alert.

The T1/T2 button is set red initially until the first change is detected. This
behaviour is to give rapid feedback that the app has detected the joystick and
can set LED colours, but will be removed in the near future.

The app runs in a console window and can be exited with Ctrl+C.

## Installation

You can run `edxlc.exe` from anywhere but it assumes you have the X52 Pro
drivers installed at the default location of
`C:\Program Files\Logitech\DirectOutput`.