# minicloze
A command-line language learning game using Tatoeba's great database. Accelerate your studies by putting your knowledge to the test in an addictive yet minimalist game.

# Features
- Support for over 400 languages
- Lean implementation
- Written in pure Rust
- Support for MacOS, Linux and Windows

# Targets
- **Long-term**
- Build a FOSS version of Clozemaster
- More gamemodes
- **Short-term**
- Get definition of a word from Wiktionary
- More user-friendly design
- Optional persistency between rounds
- Play between two non-English languages

# Usage
Just pass in the language (from www.tatoeba.org) you want to use into the prompt. If you're building locally you can pass it into `cargo run`.

# Dependencies
www.crates.io/crates/minreq

www.crates.io/crates/serde

www.crates.io/crates/rand

# Contributing
Any help is very welcome, just open a PR or an issue and I'll probably be able to reply quickly. Right now the focus is on expanding from the basic idea into a more fully-fledged and user friendly experience.

# Tatoeba
All sentences are from Tatoeba (www.tatoeba.org). Tatoeba's data is released under the CC-BY 2.0 FR license.
