# workctl

[crates.io](https://crates.io/crates/workctl) [documentation](https://docs.rs/workctl/) MIT licensed

`workctl` provides a set of higher-level abstractions for controlling concurrent/parallel programs. These abstractions are especially focused on the "controller/worker" paradigm, in which one or a few "controller" threads determine what work needs to be done and use `WorkQueues` and `SyncFlags` to communicate that to many "worker" threads.

