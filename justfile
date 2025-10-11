# https://just.systems

default:
    echo 'Hello, world!'

cargo *ARGS:
    cargo {{ARGS}}
    cargo {{ARGS}} --manifest-path rpkg/src/rust/Cargo.toml
