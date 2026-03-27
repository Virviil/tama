FROM ghcr.io/astral-sh/uv:debian AS builder
RUN uv pip install --system --target=/deps \
    duckduckgo-search>=6.0

FROM gcr.io/distroless/python3-debian12
COPY --from=builder /deps /deps
ENV PYTHONPATH=/deps
ENV PATH="/deps/bin:/usr/local/bin:/usr/bin:/bin"
COPY . /app
WORKDIR /app
ENTRYPOINT ["/app/tama", "run"]
