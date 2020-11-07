set -ex

git rev-parse HEAD | head -c 7 | awk '{ printf "%s", $0 >"commit_hash.txt" }'

cargo build --release
mkdir -p bin

if [ -f bin/dev ]; then
    mv bin/dev bin/dev-running
    cp target/release/dev bin/dev
    sudo systemctl restart dev
    rm bin/dev-running
else
    cp target/release/dev bin/dev
fi

