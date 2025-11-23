FROM python:3.11-slim

# Create non-root user
RUN groupadd -r faucet && useradd -r -g faucet faucet

WORKDIR /app

# Install dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Copy requirements and install
COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt

# Copy application
COPY app/ /app/app/

# Create data directory
RUN mkdir -p /var/zingo && chown -R faucet:faucet /var/zingo

ENV PYTHONUNBUFFERED=1 \
    PYTHONPATH=/app

USER faucet

EXPOSE 8080

HEALTHCHECK --interval=30s --timeout=5s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

CMD ["gunicorn", \
     "--bind", "0.0.0.0:8080", \
     "--workers", "4", \
     "--timeout", "60", \
     "--access-logfile", "-", \
     "--error-logfile", "-", \
     "app.main:create_app()"]