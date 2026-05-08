#!/bin/sh
# EC2 deployment entrypoint for Rust backend
# Verifies database and Redis connectivity before starting the Axum server

set -e

echo "========================================"
echo "Yomu Rust Backend - Deployment Entrypoint"
echo "========================================"

# Extract DB host from DATABASE_URL
DB_HOST=$(echo "$DATABASE_URL" | sed -n 's/.*@\([^:]*\):.*/\1/p')
DB_PORT=$(echo "$DATABASE_URL" | sed -n 's/.*:\([0-9]*\)\/.*/\1/p')
DB_HOST="${DB_HOST:-postgres}"
DB_PORT="${DB_PORT:-5432}"

# Extract Redis host from REDIS_URL
REDIS_HOST=$(echo "$REDIS_URL" | sed -n 's|redis://\([^:]*\):.*|\1|p')
REDIS_PORT=$(echo "$REDIS_URL" | sed -n 's|.*:\([0-9]*\)|\1|p')
REDIS_HOST="${REDIS_HOST:-redis}"
REDIS_PORT="${REDIS_PORT:-6379}"

echo "Database: ${DB_HOST}:${DB_PORT}"
echo "Redis:    ${REDIS_HOST}:${REDIS_PORT}"

MAX_RETRIES=30
RETRY_INTERVAL=2

# Wait for PostgreSQL
echo ""
echo "Waiting for PostgreSQL..."
for i in $(seq 1 $MAX_RETRIES); do
  if wget --spider -q "${DB_HOST}:${DB_PORT}" 2>/dev/null || nc -z "${DB_HOST}" "${DB_PORT}" 2>/dev/null; then
    echo "PostgreSQL is ready!"
    break
  fi
  echo "  Attempt $i/$MAX_RETRIES..."
  sleep $RETRY_INTERVAL
done

if [ "$i" -eq "$MAX_RETRIES" ]; then
  echo "ERROR: PostgreSQL not available"
  exit 1
fi

# Wait for Redis
echo ""
echo "Waiting for Redis..."
for i in $(seq 1 $MAX_RETRIES); do
  if wget --spider -q "${REDIS_HOST}:${REDIS_PORT}" 2>/dev/null || nc -z "${REDIS_HOST}" "${REDIS_PORT}" 2>/dev/null; then
    echo "Redis is ready!"
    break
  fi
  echo "  Attempt $i/$MAX_RETRIES..."
  sleep $RETRY_INTERVAL
done

if [ "$i" -eq "$MAX_RETRIES" ]; then
  echo "ERROR: Redis not available"
  exit 1
fi

echo ""
echo "Starting Axum server..."
echo "SQLx migrations will run automatically on startup"
echo ""

exec /app/yomu-backend-rust