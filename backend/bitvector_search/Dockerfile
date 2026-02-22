FROM nvidia/cuda:12.2.2-cudnn8-devel-ubuntu22.04 AS builder

RUN apt-get update && apt-get install -y \
    curl build-essential pkg-config openssl ca-certificates libssl-dev python3 python3-pip \
    && rm -rf /var/lib/apt/lists/*

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

WORKDIR /app

RUN pip install huggingface-hub
ENV HF_HOME=/app/model_cache
RUN python3 -c "from huggingface_hub import snapshot_download; snapshot_download('sentence-transformers/all-MiniLM-L6-v2', local_dir='/app/model_cache', local_dir_use_symlinks=False)"

COPY . .
ENV CUDA_COMPUTE_CAP=89
ENV RUSTFLAGS="-C target-cpu=haswell"
RUN cargo build --release

FROM nvidia/cuda:12.2.2-cudnn8-runtime-ubuntu22.04

RUN apt-get update && apt-get install -y \
    ca-certificates libssl3 unzip \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/bitvector_search /usr/local/bin/
COPY --from=builder /app/model_cache /app/model_cache

ENV HF_HOME=/app/model_cache
ENV HF_HUB_OFFLINE=1  

ENV RUST_LOG=info
ENV PORT=8080

CMD ["bitvector_search"]
