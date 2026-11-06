# Contributing
Some notes for myself and contributors.

## Testing
```sh
cargo build --all-targets --all-features
cargo test --all-features -- --include-ignored
cargo clippy --all-targets --all-features
cargo fmt --all --check

# try stuff
cargo run -- init --show-title --size-factor 6 --workspaces-per-row 5
cargo run -- gui --mod-key SUPER --key w 
```

Tests that need a running hyprland instance have `ignore`.


## Structure
```
src/          source tree
├─ assets/    asset files (eg. systemd-service)
├─ cli/       handle input args (using clap)
├─ client/    handle communication as the client
├─ daemon/    background daemon and GUI management
└─ handle/    event handling, switching logic
imgs/         visuals for docs and README
tests/        some tests
```


## Useful
```sh
cargo doc --all-features --open	# open documentation
```

`CTR + o`: abort and close the daemon (disables the submap)


### Update Dependecies
```sh
cargo clean
cargo update
cargo build --all-targets --all-features
```

### New Release

- update version in (Cargo.toml, PKGBUILD)
- build with: `cargo build --release` (updating the Cargo.lock)
- test binary: `./target/release/hyprswitch --version`
- commit with: `chore(main): release {version}`
- tag commit with: `git tag v{version}`
- create new release on github (upload binary)
- update AUR pkg
