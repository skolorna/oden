FROM ekidd/rust-musl-builder:beta as builder

WORKDIR /home/rust/src/oden
RUN sudo chown -R rust:rust /home/rust/src/oden
ADD . .
RUN cargo build --release --bin munin

FROM alpine:latest

RUN addgroup -S appuser \
    && adduser -S -g appuser appuser

RUN apk update \
	&& apk add --no-cache ca-certificates \
	&& rm -rf /var/cache/apk/*

WORKDIR /usr/src/app

COPY --from=builder /home/rust/src/oden/target/x86_64-unknown-linux-musl/release/munin ./

RUN chown -R appuser:appuser /usr/src/app
USER appuser

ENTRYPOINT ["./munin"]