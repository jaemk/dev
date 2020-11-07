FROM rust:1.47

# create a new empty shell
WORKDIR /working
RUN USER=root cargo new --bin dev
WORKDIR /working/dev

# copy over your manifests
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

# # this build step will cache your dependencies
RUN cargo build --release

RUN rm ./target/release/dev*
RUN rm ./target/release/deps/dev*

RUN rm ./src/*.rs

# # copy all source/static/resource files
COPY ./src ./src
COPY ./static ./static

# # build for release
RUN cargo build --release

COPY ./.git .git
RUN git rev-parse HEAD | head -c 7 | awk '{ printf "%s", $0 >"commit_hash.txt" }'
RUN rm -rf .git

# set the startup command to run your binary
CMD ["./target/release/dev"]
