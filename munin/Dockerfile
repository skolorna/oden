FROM clux/muslrust:1.66.1 as build-env
WORKDIR /app
COPY . .
RUN cargo build --release --bin munin

FROM gcr.io/distroless/static:nonroot
COPY --from=build-env --chown=nonroot:nonroot /app/target/x86_64-unknown-linux-musl/release/munin /
ENTRYPOINT ["/munin"]
