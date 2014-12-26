img-dup
=======

A tool for finding duplicate and similar images in directory trees, written in Rust

Powered by https://github.com/cybergeek94/img_hash

Usage
=====

Launch GUI:
```shell
# Binary
./img_dup -g

# Running with Cargo
cargo run -- -g
```

Scan local directory in CLI:
```shell
# Binary
./img_dup

# Cargo
cargo run
```

For a guide on the Graphical User Interface, see `GUI.md` in this repository.

For information on the command line flags, see `CLI.md` in this repository.

Building
========

Ubuntu-based:
```shell
sudo apt-get install libfreetype6-dev libsdl2-dev

git clone https://github.com/cybergeek94/img_dup
cd img_dup
cargo build
```

License
=======

This software is GPL-licensed, with several MIT-licensed dependencies.

Please see `LICENSE.md` in this repository.

GPL-Licensed Font
=================
This program uses the GPL-licensed `FreeSerif.otf` font, copied, unmodified, from the [GNU Freefont][1] distribution.
Since this program is GPL, it is free to include GPL-licensed fonts.

[1]: https://www.gnu.org/software/freefont/index.html
