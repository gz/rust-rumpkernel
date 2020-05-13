[![Build Status](https://travis-ci.org/gz/rust-rumpkernel.svg)](https://travis-ci.org/gz/rust-rumpkernel)

# Rumpkernel

Builds rumpkernel sources as a rust crate for convenient use in your rust project.

## build.rs

The build roughly the following steps as part of the `build.rs` script:

```bash
git clone https://github.com/gz/rumprun.git
git checkout netbsd-8
git submodule update --init --depth 1
./build-rr.sh -j12 bespin -- -F "CFLAGS=-w"
```
