FROM rust:alpine AS build-stage

RUN apk update
RUN apk add cmake make musl-dev g++ perl

WORKDIR /build
COPY Cargo.toml ./
COPY Cargo.lock ./
COPY src ./src
COPY resources ./resources

RUN cargo build --release

# Build image from scratch
FROM scratch
LABEL org.opencontainers.image.source="https://github.com/diz-unimr/kafka-consent-to-onkostar"
LABEL org.opencontainers.image.licenses="AGPL-3.0-or-later"
LABEL org.opencontainers.image.description="Send 'consent-json-idat information to Onkostar"

COPY --from=build-stage /build/target/release/kafka-consent-to-onkostar .
USER 65532

CMD ["./kafka-consent-to-onkostar"]
