set -ex

git rev-parse HEAD | head -c 7 | awk '{ printf "%s", $0 >"commit_hash.txt" }'

cargo build --release
mkdir -p bin
cp target/release/dev bin/dev

