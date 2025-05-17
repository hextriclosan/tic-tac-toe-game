# --- Stage 1: Chef base for dependency caching ---
FROM lukemathwalker/cargo-chef:latest-rust-1 AS chef
WORKDIR /app

# --- Stage 2: Plan dependencies ---
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# --- Stage 3: Build dependencies WASM and server ---
FROM chef AS builder
# Load cached dependencies
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

# Copy full source
COPY . .

# Build the WASM frontend using wasm-pack
RUN cargo install wasm-pack
RUN wasm-pack build game --target web

# Copy static artifacts
RUN mkdir -p tic-tac-toe-game/static && \
    cp game/pkg/game.js tic-tac-toe-game/static/ && \
    cp game/pkg/game_bg.wasm tic-tac-toe-game/static/ && \
    cp static/index.html tic-tac-toe-game/static/

# Build backend binary
RUN cargo build --release --bin tic-tac-toe-game

# --- Stage 4: Runtime container ---
FROM debian:bookworm-slim AS runtime
WORKDIR /app

# Copy binary
COPY --from=builder /app/target/release/tic-tac-toe-game /usr/local/bin

# Copy static assets
COPY --from=builder /app/tic-tac-toe-game/static ./static

# Set entrypoint
ENTRYPOINT ["/usr/local/bin/tic-tac-toe-game"]
