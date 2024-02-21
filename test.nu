echo "" | save -f ./target/debug/test/log.txt
cargo test -- --test-threads=1
