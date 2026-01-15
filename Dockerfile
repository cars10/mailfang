# Stage 1: Frontend builder
FROM node:24-alpine AS frontend-builder

WORKDIR /app/frontend

COPY frontend/package*.json ./

RUN npm ci --only=production=false

COPY frontend/ ./

RUN npm run build


# Stage 2: Build Rust backend
FROM rust:1.91-alpine AS backend-builder

RUN apk add --no-cache musl-dev sqlite-dev

WORKDIR /app/backend

COPY backend/Cargo.toml backend/Cargo.lock ./

RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    cargo build --release && \
    rm -rf src

COPY backend/src ./src
COPY backend/migrations ./migrations

COPY --from=frontend-builder /app/frontend/dist /app/frontend/dist

RUN cargo build --release --features embed-frontend


# Stage 3: Runtime image
FROM alpine:latest AS runtime

RUN apk add --no-cache \
    ca-certificates \
    sqlite \
    curl \
    && rm -rf /var/cache/apk/*

RUN addgroup -g 1000 appuser && \
    adduser -D -u 1000 -G appuser appuser

COPY --from=backend-builder /app/backend/target/release/mailfang /usr/bin/mailfang

RUN mkdir -p /data && \
    touch /data/mailfang.db && \
    chown -R appuser:appuser /data

WORKDIR /data

USER appuser

ENV DATABASE_URL=sqlite:///data/mailfang.db
ENV SMTP_HOST=0.0.0.0:2525
ENV WEB_HOST=0.0.0.0:3000
ENV WEB_PORT=3000
ENV SMTP_PORT=2525

CMD ["mailfang"]
