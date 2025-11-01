# Compilateur en chef
FROM lukemathwalker/cargo-chef:latest-rust-1 AS chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --recipe-path recipe.json
# Build application
COPY . .
RUN cargo build --release --bin lexique3-api

# On n'a plus besoin de la toolchain, ça run tout seul.
FROM debian:bookworm-slim AS runtime
WORKDIR /app
# On copie simplement le bin et le fichier csv
COPY --from=builder /app/target/release/lexique3-api /app/lexique3-api
COPY Lexique383.tsv /app/Lexique383.tsv

#On définfit l'netrypoint là ou on a placé le bin
ENTRYPOINT ["/app/lexique3-api"]
