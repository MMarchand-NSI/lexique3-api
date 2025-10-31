# Utiliser cargo-chef pour le caching des dépendances
FROM rust:1.82-slim as chef
WORKDIR /app
RUN cargo install cargo-chef

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json

# Construire les dépendances (cette couche sera cachée)
RUN cargo chef cook --release --recipe-path recipe.json

# Construire l'application
COPY . .
RUN cargo build --release

# Image finale minimale
FROM debian:bookworm-slim AS runtime
WORKDIR /app

# Installer les dépendances système nécessaires
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copier le binaire
COPY --from=builder /app/target/release/lexique3-api /app/lexique3-api

# Copier les données Lexique3 (décommenter si vous avez le fichier)
# COPY Lexique383.tsv /app/

ENV RUST_LOG=info
ENV PORT=8080

EXPOSE 8080

CMD ["/app/lexique3-api"]