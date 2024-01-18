# ServerSeeker API+
A wrapper for the ServerSeeker API that adds more features

# Requirements
To compile this project, you need the Rust toolchain. To get it, install rustup from https://rustup.rs/.

This is not required you run it in Docker.

# Usage
By default, the port is 3000, to change this, modify the `port` variable in src/main.rs

Running:

`cargo run --release`

Building:

`cargo build --release # The binary will be under target/release/ssapi-plus`

Running in Docker:

```
docker build . -t ssapi-plus
docker run -p 3000:3000 --name ssapi-plus ssapi-plus
```
