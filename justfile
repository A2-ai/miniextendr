# https://just.systems

default:
    echo 'Hello, world!'

cargo fmt:
    cargo fmt
    cargo fmt --manifest-path rpkg/src/rust/Cargo.toml