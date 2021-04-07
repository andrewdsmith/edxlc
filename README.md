# **E**lite **D**angerous **X**52 Pro **L**ED **C**ontrol - EDXLC

The aim of this application is to automatically control the button LEDs on your
X52 Pro joystick using the state of your ship or SRV in the game Elite
Dangerous.

The current version only detects your ship's landing gear deployed status and
indicates this on the T1/T2 button, which is the default button. For testing,
the button is red initially, then turns yellow if the landing gear is deployed
and green if it is retracted.

The application runs in a console window and can be killed with Ctrl + C but
will take 30 seconds after this to shut down.

## Installation

You can run `edxlc.exe` from anywhere but it assumes you have the X52 Pro
drivers installed at the default location of
`C:\Program Files\Logitech\DirectOutput`.