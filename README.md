# Unofficial Rust implementation of M5Stack Avatar

## Overview

This is a unofficial Rust implementation of M5Stack Avatar, which draws avatar face on LCD displays.

## How to run

On Linux system, you can run the example on your machine.

```
cargo run --example mock --features std
```

## License

MIT or Apache 2.0
The balloon text is rendered with Japanese bitmap fonts from the
[u8g2](https://github.com/olikraus/u8g2) project (`b12`/`b16` `t_japanese3`,
via the [u8g2-fonts](https://crates.io/crates/u8g2-fonts) crate). These glyphs
originate from the /efont/ unicode bitmap fonts — primarily the public-domain
Shinonome and jiskan fonts.
