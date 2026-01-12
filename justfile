RUST_LOG := "debug"

alias w := watch

# Watches the server and recompiles it when changes are made
watch:
    RUST_LOG={{RUST_LOG}} cargo watch -q -c -x 'run -q'