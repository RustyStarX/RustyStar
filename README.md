# RustyStar

EnergyStar alernative in Rust

## Goals

- [x] Multi Window support
- [ ] Child processes support
  - [x] Direct child processes
  - [ ] Child process of child processes
- [x] Event-based foreground window boost
- [ ] Event-based blacklist throttle for non-GUI processes
  > currently implemented in [fitgirl-ecoqos](https://github.com/mokurin000/fitgirl-ecoqos)
- [ ] recover processes on exit
  - [x] Ctrl-C handle
  - [ ] windows terminate handle

## Non-goals (for now)

- Power mode aware, e.g. pause & recover on AC supply
- Useless bloated GUI
- Extremely lightweight binary (never prioritier than project maintainability)

## Status

Coming soon™
