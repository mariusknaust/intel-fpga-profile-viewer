# intel-fpga-profile-viewer
Unofficial CLI to aggregate and display profiling information recorded by the Intel FPGA Dynamic Profiler.

Serves as a command line alternative to Intel VTune for viewing Intel FPGA Dynamic Profiler JSON-files, especially when working on a remote machine.

## State
Currently only tested with a few profile files of the `2021.2.0.268.1` release out of the Intel oneAPI packages.
Issues (optional with profile files attached) or even pull request are welcome.

## Build
Builds with stable Rust (e.g. via [rustup](https://www.rust-lang.org/tools/install)):
```
cargo build --release
```

## Options
It is possible to expand the module instance section with `--expand` or select the kernels to be considered by supplying their names to `--kernels`.
