cargo run --quiet -- $1 && clang output.ll -o ritual_exec -w && ./ritual_exec
