FROM rust:1.83-alpine3.21 AS builder
WORKDIR /usr/src/smart-hot-water-tank
RUN apk add --no-cache musl-dev
COPY . .
RUN cargo install --path .

FROM alpine:3.21
COPY --from=builder /usr/local/cargo/bin/smart-hot-water-tank /usr/local/bin/smart-hot-water-tank 
CMD ["/usr/local/bin/smart-hot-water-tank"]