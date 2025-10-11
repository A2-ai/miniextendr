# https://just.systems

default:
    echo 'Hello, world!'

configure:
    autoreconf -vif rpkg

cargo *ARGS:
    cargo {{ARGS}}
    cargo {{ARGS}} --manifest-path rpkg/src/rust/Cargo.toml
