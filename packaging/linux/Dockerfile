FROM node:20-bookworm

ARG DEBIAN_FRONTEND=noninteractive
ARG RUST_TOOLCHAIN=stable

RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        build-essential \
        ca-certificates \
        clang \
        cmake \
        curl \
        file \
        fuse \
        git \
        libappindicator3-dev \
        libasound2-dev \
        libfuse2 \
        libgtk-3-dev \
        libopenblas-dev \
        librsvg2-dev \
        libssl-dev \
        libwebkit2gtk-4.1-dev \
        patchelf \
        pkg-config \
        wget \
        xz-utils \
    && rm -rf /var/lib/apt/lists/*

RUN corepack enable \
    && corepack prepare pnpm@10.33.0 --activate \
    && curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
        | sh -s -- -y --profile minimal --default-toolchain "${RUST_TOOLCHAIN}"

ENV PATH="/root/.cargo/bin:${PATH}"
ENV NO_STRIP=true
ENV BLAS_INCLUDE_DIRS=/usr/include/x86_64-linux-gnu
ENV OPENBLAS_PATH=/usr/lib/x86_64-linux-gnu

WORKDIR /workspace
COPY . .

RUN pnpm --dir frontend install --frozen-lockfile

ENTRYPOINT ["/workspace/packaging/linux/build.sh"]
