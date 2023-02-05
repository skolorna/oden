ARG RUST_VERSION=1.67.0

FROM clux/muslrust:$RUST_VERSION AS planner
RUN cargo install cargo-chef
COPY . .
RUN cargo chef prepare --bin munin --recipe-path recipe.json

FROM clux/muslrust:$RUST_VERSION AS cacher
RUN cargo install cargo-chef
COPY --from=planner /volume/recipe.json recipe.json
RUN cargo chef cook --bin munin --release --target x86_64-unknown-linux-musl --recipe-path recipe.json

FROM clux/muslrust:$RUST_VERSION AS builder
COPY . .
COPY --from=cacher /volume/target target
COPY --from=cacher /root/.cargo /root/.cargo
RUN cargo build --bin munin --release

FROM gcr.io/distroless/static
COPY --from=builder /volume/target/x86_64-unknown-linux-musl/release/munin /munin
ENTRYPOINT ["/munin"]
