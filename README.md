# workctl

[![Information on crates.io](https://img.shields.io/crates/v/workctl.svg)](https://crates.io/crates/workctl)
[![Documentation on docs.rs](https://docs.rs/workctl/badge.svg)](https://docs.rs/workctl/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)


`workctl` provides a set of higher-level abstractions for controlling
concurrent/parallel programs.
These abstractions are especially focused on the "controller/worker" paradigm,
in which one or a few "controller" threads determine what work needs to be done
and use `WorkQueues` and `SyncFlags` to communicate that to many "worker" threads.

