# EngCalc

EngCalc is a terminal scientific calculator focused on engineering workflows. It
supports variables, user-defined functions, constants, unit conversion, compound
units, history, autocomplete, and a Ratatui-based interface.

## Run

```sh
cargo run
```

Build an optimized binary:

```sh
cargo build --release
```

Run the test suite:

```sh
cargo test
```

## Examples

```text
2 + 3 * 4
sin(pi / 2)
r = 3
pi * r^2
10 km in m
36 km/h in m/s
1 kg * 1 m/s^2
f(x) = x^2 + 2x + 1
f(3)
simpson(f, 0, 1, 100)
quadratic(1, -3, 2)
```

## Commands

Commands start with `:`.

```text
:help
:clear
:vars
:consts
:history
:clearhist
:quit
```

## Keys

```text
Enter    Evaluate input
Tab      Accept autocomplete suggestion
Up/Down  Navigate history or autocomplete
Esc      Clear input or close overlays
Ctrl+C   Quit
Ctrl+L   Clear screen
Ctrl+U   Clear input
F1       Help
F2       Constants
F3       Clear all
F4       Functions
```

## Notes

- Trigonometric functions use radians.
- Unit-aware addition and subtraction require compatible dimensions.
- The workspace is restored from the latest history entry on startup.
- History is stored under the platform local data directory in `engcalc/history.json`.
