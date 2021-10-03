# Changes

## Version 1.11

- Do not show FSD as active when in supercruise

## Version 1.10

- Supports active state for night vision control
- Supports alternative (optional) configuration when night vision on
- Supports all combinations of two colours for flashing light modes in
  configuration
- Change default active light mode to `amber-flash` to make this state more
  noticeable
- Supports specifying the configuration file as a command line argument
- Make alternative hardpoints deployed configuration optional in the
  configuration file; falls back to default configuration from the file if
  missing

## Version 1.9

- Supports controls bound to the POV2 hat and throttle
- Shows alert on throttle control when speeding
- Sets all lights to the inactive state even if the are not mapped to recognised
  controls; this is noticable for example when hardpoints are deployed using the
  default configuration
- Keep boolean lights on when inactive by default as this gives a better
  experience with both the Fire button and the throttle
- Prevent hardpoints deployed being the global state when in supercruise; this
  could be triggered previously by using the FSS

## Version 1.8

- Supports alternative configuration when hardpoints are deployed
- **Important**: Old configuration files are now invalid; see README for
  details on the new format; you can delete or rename your old file and let
  the app create a new, valid file
- Shows blocked state for FSD and boost when landing gear deployed
- Correctly shows alert state for FSD when the FSD is charging and the ship is
  both in supercruise and overheating (already worked properly in normal flight)

## Version 1.7

- Supports `off` mode for red/amber/green lights in configuration
- Supports the Fire button and its boolean light
- Supports both boolean and red/amber/green light modes in configuration -
  **Important**: Old configuration files are now invalid; see README for
  details on the new format; you can delete or rename your old file and let
  the app create a new, valid file
- Shows active state for hardpoints when deployed and inactive state when not
  deployed
- Shows active state for FSD when in supercruise
- Shows blocked state for FSD when hardpoints are deployed (unless in
  supercruise)

## Version 1.6

- Reads control bindings from Odyssey `Custom.4.0.binds` file
- Reads configuration from `edxlc.toml` file in the current working directory
- Writes default `edxlc.toml` configuration file if missing
- Shows alert state for landing gear if not deployed while docking

## Version 1.5

- Shows active state (yellow) on silent running buttons
- Shows alert state (red/yellow flashing) on FSD and silent running buttons if
  that control is active but ship is also overheating
- Shows correct states immediately, i.e. do not wait for first significant state
  change before updating.
- Shows correct states consitently on buttons where one LED represents multiple
  bound controls, e.g. on T1 and T2

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
