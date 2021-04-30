# Changes

## Version 1.5

- Show correct states immediately, i.e. do not wait for first significant state
  change before updating.

## Version 1.4

- App icon!
- Shows alert state (red/yellow flashing) on heat sink button when overheating
- Silences most output but this can be re-enabled by setting the `RUST_LOG`
  environment variable to `edxlc=debug` prior to execution

## Version 1.3

- Shows active state (yellow) on all hyperspace and supercruise related buttons
  when FSD is charging
- Shows blocked state (red) on all hyperspace and supercruise related buttons
  when mass-locked or during FSD cooldown
- Correctly displays an active state (yellow) where one LED represents multiple
  bound controls, e.g. on T1 and T2

## Version 1.2

- Reads button bindings from file `Custom.3.0.binds` so that landing gear,
  cargo scoop, and external light states are displayed on the correct, user
  configured buttons
- Adds Clutch, Fire A, Fire B, Fire D, Fire E, T1, T3 and T5 to the supported
  buttons

## Version 1.1

- Sets T3/T4 button yellow when ship cargo scoop lowered
- Sets T5/T6 button yellow when ship external lights on
- Displays version number on startup
- Exits immediately on Ctrl+C

## Version 1.0

- Sets T1/2 button yellow when ship landing gear deployed
