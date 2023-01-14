FROM clux/muslrust:1.66.0 as build-env
WORKDIR /app
COPY . .
RUN cargo build --release --bin hugin

FROM gcr.io/distroless/static:nonroot
COPY --from=build-env --chown=nonroot:nonroot /app/target/x86_64-unknown-linux-musl/release/hugin /
EXPOSE 8000
ENTRYPOINT ["/hugin"]
