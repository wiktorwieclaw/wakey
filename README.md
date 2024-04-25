# Wakey
Physical alarm clock controlled via the Internet.

## Prerequisites
* `xtensa-esp32-none-elf` target toolchain
    ```
    cargo install espup
    espup install
    ```
* [cargo-espflash](https://github.com/esp-rs/espflash)
    ```
    cargo install cargo-espflash
    ```

## Flashing
Use `flash` alias from `.cargo\config.toml`.
```
cargo flash
```