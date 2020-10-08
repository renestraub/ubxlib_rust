# Simple build script for target release image

# cargo clean --target=arm-unknown-linux-gnueabihf --release
cargo build --target=arm-unknown-linux-gnueabihf --release
cargo bloat --release -n 10

arm-linux-gnueabihf-strip target/arm-unknown-linux-gnueabihf/release/gnss-mgr
echo
ls target/arm-unknown-linux-gnueabihf/release/gnss-mgr -alh
