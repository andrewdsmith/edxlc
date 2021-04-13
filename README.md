# Elite Dangerous X52 Pro LED Control - EDXLC

The aim of this small companion app is to automatically control the button LEDs
on your Saitek X52 Pro joystick using the state of your ship or SRV in the game
Elite Dangerous.

Currently the app toggles the LEDs on the default buttons for:

- Landing gear (T1/T2)
- Cargo scoop (T3/T4)
- External lights (T5/T6)

The intent is that over time the app will read the bindings you have configured
for buttons and toggle the associated button LEDs automatically without manual
configuration. Additionally, the app will become configurable to set the desired
colours for conditions such as active, in use and alert.

The T1/T2 button is set red initially until the first change is detected. This
behaviour is to give rapid feedback that the app has detected the joystick and
can set LED colours, but will be removed in the near future.

The app runs in a console window and can be exited with Ctrl+C.

## Installation

You can run `edxlc.exe` from anywhere but it assumes you have the X52 Pro
drivers installed at the default location of
`C:\Program Files\Logitech\DirectOutput`.