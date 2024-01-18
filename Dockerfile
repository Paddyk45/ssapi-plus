FROM rustlang/rust:nightly-bullseye-slim
WORKDIR /opt/build
COPY . .
RUN cargo build --release
WORKDIR /opt/ssap
RUN cp /opt/build/target/release/ssapi-plus .
ENTRYPOINT ["/opt/ssap/ssapi-plus"]
EXPOSE 3000/tcp