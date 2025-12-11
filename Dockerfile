# Stage 1: Frontend builder
FROM node:20-alpine AS frontend-builder

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

RUN cargo build --release


# Stage 3: Runtime image
FROM alpine:latest AS runtime

RUN apk add --no-cache \
    ca-certificates \
    sqlite \
    curl \
    && rm -rf /var/cache/apk/*

RUN addgroup -g 1000 appuser && \
    adduser -D -u 1000 -G appuser appuser

WORKDIR /app

COPY --from=backend-builder /app/backend/target/release/mailfang-backend /app/mailfang-backend

COPY --from=frontend-builder /app/frontend/dist /app/static

RUN mkdir -p /data && \
    touch /data/mailfang.db && \
    chown -R appuser:appuser /app /data

USER appuser

EXPOSE 3000 2525

ENV STATIC_DIR=/app/static
ENV DATABASE_URL=sqlite:///data/mailfang.db
ENV WEB_PORT=3000
ENV SMTP_PORT=2525

HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:3000/health || exit 1

CMD ["/app/mailfang-backend"]
