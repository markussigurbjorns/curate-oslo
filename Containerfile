FROM rust:1.82 AS builder

WORKDIR /app

#RUN git clone https://github.com/markussigurbjorns/curate-oslo.git /app
COPY ./ /app

RUN cargo build --release

WORKDIR /app/wasm-frontend

RUN curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

RUN wasm-pack build --target web

FROM registry.fedoraproject.org/fedora-minimal:latest

WORKDIR /app

RUN mkdir -p /app/public
RUN mkdir -p /app/public/wasm


COPY --from=builder /app/target/release/curate-oslo /app/server
COPY --from=builder /app/wasm-frontend/pkg /app/public/wasm/
COPY --from=builder /app/public /app/public/

EXPOSE 6969
EXPOSE 6868

CMD ["/app/server"]
