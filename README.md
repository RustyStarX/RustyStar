# RustyStar

EnergyStar alernative in Rust.

## Suggested CPU

> [reference](https://devblogs.microsoft.com/performance-diagnostics/introducing-ecoqos/#supported-hardware)

- Intel: 10gen or newer (not 12gen)
- AMD: Ryzen 5000 Series or newer
- Qualcomm: Basically all

## I only want to throttle blacklisted ones

RustyStar now supports configuration.

Locate to `%AppData%/RustyStarX/RustyStar/config` and open `config.toml`,

disable `listen_foreground_events` and switch mode of `listen_new_process` to `"blacklist_only"`, configure the blacklist.

Check [wiki page of configuration](https://github.com/RustyStarX/RustyStar/wiki/Configuration) for more details.

## Roadmap

- [x] Multi Window support
- [x] Child processes support
  - [x] Direct child processes
  - [x] Process tree walking
- [x] Event-based foreground window boost
- [x] Event-based throttle for all new processes
- [x] Recover processes on exit
  - [x] Ctrl-C handle
  - [ ] windows wm_close event handle
- [x] Support UWP applications
- [x] Support `SYSTEM` privileged processes
  > You must run `RustyStar.exe` as administrator to throttle them!
- [x] Configurable whitelist and blacklist

## What is the efficiency mode?

The "Efficiency mode" in task manager does two things on a process:

- Enable EcoQoS
- Change base priority to IDLE

The latter is not always useful, especially when we throttle mostly all processes.
And the former requires hardware support to perform the best effect.

[`EcoQoS`](https://devblogs.microsoft.com/performance-diagnostics/introducing-ecoqos/) was
first introduced in Windows 10 Preview Build 21359.

The main impact from EcoQoS are:
  - Energy efficiency & Sustainability
  - Reduced heat and fan noise (where this project actually care)

## What does EcoQoS actually do?

According to [MSDN](https://learn.microsoft.com/en-us/windows/win32/procthread/quality-of-service#quality-of-service-levels),
`Eco` QoS level means:

> Always selects most efficient CPU frequency and schedules

## Non-goals

- Useless bloated GUI
- Power mode awareness, e.g. pause & recover on AC supply
- Extremely lightweight binary
