clear
printf "\nintergate build started...  %s\n" "$(date -u -Iseconds)"
cargo fmt
RUSTFLAGS="-C opt-level=z" cargo build --release
printf "intergate build finished.  %s\n" "$(date -u -Iseconds)"


