# RustyStar

EnergyStar alernative in Rust.

## Roadmap

- [x] Multi Window support
- [x] Child processes support
  - [x] Direct child processes
  - [ ] Process tree walking
- [x] Event-based foreground window boost
- [x] Event-based throttle for all new processes
- [x] Recover processes on exit
  - [x] Ctrl-C handle
  - [ ] windows terminate handle
- [ ] Support UWP applications (`EnumChildWindows`)
- [ ] Configurable whitelist and blacklist

## Non-goals (for now)

- Power mode aware, e.g. pause & recover on AC supply
- Useless bloated GUI
- Extremely lightweight binary (never prioritier than project maintainability)

## Status

Partially functional.
