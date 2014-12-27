`img_dup`
=======

A tool for finding duplicate and similar images in directory trees, written in Rust

Powered by: 
* https://github.com/rust-lang/rust

* https://github.com/PistonDevelopers/conrod

* https://github.com/cybergeek94/img_hash

Usage
=====
Scan local directory in CLI:
```shell
# Binary
cargo build
./img_dup

# Cargo
cargo run
```

Launch GUI (requires `img_dup` to be compiled with GUI support):
```shell
# Binary
./img_dup -g

# Cargo
cargo run --features="gui" -- -g
```

For a guide on the Graphical User Interface, see `GUI.md` in this repository.

For information on the command line flags, see `CLI.md` in this repository.

Building
========

`img_dup` is built without GUI support by default for compatibility with CLI-only systems. It will not pull in SDL2 or Freetype and doesn't require them to build or run if compiled without GUI support. See the next section for building with GUI support.

```shell
git clone https://github.com/cybergeek94/img_dup
cd img_dup
cargo build
```

Building with GUI support
==========================

####Prerequisites
```shell
sudo apt-get install libfreetype6-dev libsdl2-dev
```
####Building the GUI
Pass the `--features="gui"` flag to Cargo:
```shell
cargo build --features="gui"

#OR

cargo run --features="gui" -- -g
```

TODO
====
* UI cleanups/improvements
* Windows, Mac, Linux binary packages
* Launchpad PPA

License
=======

This software is GPL-licensed, with several MIT-licensed dependencies.

Please see `LICENSE.md` in this repository.

GPL-Licensed Font
=================
This program uses the GPL-licensed `FreeSerif.otf` font, unmodified, from the [GNU Freefont][1] distribution.
[1]: https://www.gnu.org/software/freefont/index.html
