RUST_LOG := "debug"

alias w := watch

watch:
    RUST_LOG={{RUST_LOG}} cargo watch -q -c -x 'run -q'