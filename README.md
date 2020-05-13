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

## Interesting defines (in build-rr.sh)

<https://ftp.netbsd.org/pub/NetBSD/NetBSD-current/src/sys/rump/README.compileopts>

`RUMP_CURLWP=hypercall` (slower) or `RUMP_CURLWP=__thread` (faster)

Works on >1 cores:
`RUMP_DIAGNOSTIC`

Doesn't work with cores > 1:
`RUMP_LOCKDEBUG`
`RUMP_DEBUG`
