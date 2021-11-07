FROM ekidd/rust-musl-builder:beta as base

WORKDIR /home/rust/src/butler
RUN sudo chown -R rust:rust /home/rust/src/butler
ADD . .
RUN cargo build --release

FROM alpine:latest AS build-http

RUN addgroup -S appuser \
    && adduser -S -g appuser appuser

RUN apk update \
	&& apk add --no-cache ca-certificates \
	&& rm -rf /var/cache/apk/*

WORKDIR /usr/src/app

COPY --from=base /home/rust/src/butler/target/x86_64-unknown-linux-musl/release/butler-http ./

RUN chown -R appuser:appuser /usr/src/app
USER appuser

EXPOSE 8000
CMD ["./butler-http"]

FROM alpine:latest AS build-indexer

RUN addgroup -S appuser \
    && adduser -S -g appuser appuser

RUN apk update \
	&& apk add --no-cache ca-certificates \
	&& rm -rf /var/cache/apk/*

WORKDIR /usr/src/app

COPY --from=base /home/rust/src/butler/target/x86_64-unknown-linux-musl/release/butler-indexer ./

RUN chown -R appuser:appuser /usr/src/app
USER appuser

CMD ["./butler-indexer"]
