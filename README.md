# tranquil

[![Crates.io](https://img.shields.io/crates/v/tranquil.svg)](https://crates.io/crates/tranquil)
[![MIT](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/Possseidon/tranquil/blob/main/LICENSE)
[![Crates.io](https://img.shields.io/crates/d/tranquil.svg)](https://crates.io/crates/tranquil)
[![CI status](https://github.com/Possseidon/tranquil/actions/workflows/rust.yml/badge.svg)](https://github.com/Possseidon/tranquil/actions/workflows/rust.yml?query=branch%3Amain+)

A framework for [Discord](https://discord.com/) bots based on [serenity](https://github.com/serenity-rs/serenity).

Similar to [poise](https://github.com/serenity-rs/poise) but with some different design decisions. Most notably, it uses a `derive` based approach on `enum`s to define entire command groups rather than using free functions with attributes. Additionally, to keep things simple, tranquil does **not** support parsing of old school prefix-commands; only slash commands are supported.
