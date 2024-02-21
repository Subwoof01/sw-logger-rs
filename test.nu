echo "" | save -f ./target/debug/test/test.log
echo "" | save -f ./target/debug/test/custom.log
cargo test -- --test-threads=1
