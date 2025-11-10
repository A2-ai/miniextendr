# https://just.systems

default:
    echo 'Hello, world!'

configure:
    autoreconf -vif rpkg

cargo-doc:
    cargo doc --document-private-items --lib --target-dir rpkg/src/rust/target
    cargo doc --document-private-items --lib --bin document --manifest-path rpkg/src/rust/Cargo.toml --target-dir rpkg/src/rust/target --open

cargo-expand:
    cargo expand --lib --manifest-path rpkg/src/rust/Cargo.toml

cargo *ARGS:
    cargo {{ARGS}}
    cargo {{ARGS}} --manifest-path rpkg/src/rust/Cargo.toml

test-r-build:
    R CMD build --compression=none rpkg
    tar -xvzf rpkg_0.0.0.9000.tar -C "$(mkdir -p rpkg_build && echo rpkg_build)"