# Steam auto gamemode

A small application that watches for processes belonging to Steam games, and enables [Feral's gamemode](https://github.com/FeralInteractive/gamemode) for those processes.

## Detection

Detection relies the "STEAM_GAME" custom X property, which should be set by the Steam overlay, but appears to be set even when the overlay is disabled. 

It only works on X windows, it will not work for Wayland windows, though it might for XWayland (untested.)

## Runtime dependencies

It uses D-Bus and X11. The required client libs should be present by default on any desktop install.

To install gamemode see its [README](https://github.com/FeralInteractive/gamemode).

## Building

### Local

Tested with Rust 1.46.0. It requires cargo and rustc. 

The Rust project recommends [Rustup](https://rustup.rs/) for installation, but your distribution's Rust packages will work as well if they have the right versions.
On Debian and derivatives, installing the `cargo` package will pull all the necessary packages as dependencies.

Building the Rust D-Bus crate requires the dev package, eg. `libdbus-1-dev` on Debian and derivatives. 

```sh
cargo build --release
```

The output binary can be found in the `target/release` subdirectory.

### Docker

Requires docker >= 18.09

```sh
./build.sh docker
```

The output binary can be found in the `docker_target` subdirectory.

Note the docker build will link against Debian buster's D-Bus client libraries. The resulting binary will probably work fine in Debian and derivatives, but to guarantee compatiblity you should build locally to link against your distribution's libdbus. 

(D-Bus's ABI policy provides backwards compatiblity without relinking, but I have not put this policy to the test, and distributions are more than capable of ruining upstream's ABI compatibility.)

## TODO

- Wayland support
- Detect games from other launchers
- Other ways of detecting game processes without relying on launchers
