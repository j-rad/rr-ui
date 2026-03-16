# Stage 1: Build the SvelteKit frontend
FROM node:18-slim as frontend
WORKDIR /app/web
COPY web/package*.json ./
RUN npm install
COPY web/ .
RUN npm run build

# Stage 2: Build the Rust backend
FROM rust:1-slim-bullseye as backend
WORKDIR /app
COPY . .
RUN apt-get update && apt-get install -y protobuf-compiler && rm -rf /var/lib/apt/lists/*
RUN cargo build --release --features server

# Stage 3: Create the runtime image
FROM debian:bullseye-slim
WORKDIR /usr/local/rr-ui
COPY --from=backend /app/target/release/rr-ui .
COPY --from=frontend /app/web/build ./web/build
EXPOSE 2053
ENTRYPOINT ["./rr-ui", "run", "--port", "2053"]
