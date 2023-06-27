@echo Building Rust project for Windows in release mode optimized for size and statically linked to the C runtime.
set "RUSTFLAGS=-C target-feature=+crt-static"
cargo build --profile release-small --no-default-features
rcedit "D:/mike/rust/musical_bindings/target/release-small/musical_bindings.exe" --set-icon "icon.ico"
pause