rucd
====
A command line tool to browse and search the Unicode character database.

[![Linux build status](https://api.travis-ci.org/BurntSushi/rucd.png)](https://travis-ci.org/BurntSushi/rucd)
[![Windows build status](https://ci.appveyor.com/api/projects/status/github/BurntSushi/rucd?svg=true)](https://ci.appveyor.com/project/BurntSushi/rucd)
[![](http://meritbadge.herokuapp.com/rucd)](https://crates.io/crates/rucd)

Dual-licensed under MIT or the [UNLICENSE](http://unlicense.org).

### Installation

Please note that this command is currently a **work in progress**.

While it's a work in progress, to try it, you should clone this repository
and build it:

```
$ git clone git://github.com/BurntSushi/rucd
$ cd rucd
$ cargo build --release
$ ./target/release/rucd --help
```

### Motivation

My line of work tends to lead me to ask questions about Unicode, its codepoints
and their properties. There are various command line tools out there already
that are service in pursuit of answering those questions, but they don't quite
do what I want. With that said, the **primary motivator** for building this
tool was to act as a forcing function for me to become more intimately familiar
with Unicode and its various intricacies.
