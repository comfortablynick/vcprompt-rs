#!/usr/bin/env just --justfile
bin_name := 'vcprompt-rs'
dev := '0'

alias r := run
alias b := build
alias i := install
alias h := help

# build release binary
build:
    cargo build --release

# build release binary ONLY during dev; otherwise install
install:
    #!/usr/bin/env bash
    if [[ {{dev}} -eq "1" ]]; then
        cargo run --release
    else
        cargo install --path . -f
    fi #

# build release binary and run
run:
    cargo run --release #

help:
    ./target/release/{{bin_name}} -h

# run release binary
rb +args='':
    ./target/release/{{bin_name}} {{args}}

bench:
    hyperfine -w 100 'vcprompt-rs ~/src/neovim' 'vctest -f "%b %r %p %u %m" ~/src/neovim'

test:
    cargo test

fix:
    cargo fix
