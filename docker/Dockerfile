# syntax=docker/dockerfile:1.7
# Rust SDK container used for deterministic build/test and cross-RID publish.
FROM rust:1-bookworm AS build

RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        pkg-config libxkbcommon-dev libfontconfig1-dev libfreetype6 libgl1 libegl1 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /src
COPY . .
RUN rustup target add x86_64-pc-windows-gnu && cargo fetch

FROM build AS test
RUN cargo test --workspace

FROM build AS publish-windows
RUN cargo build --release -p greg-app -p greg-cli --target x86_64-pc-windows-gnu

FROM build AS publish-macos-x64
RUN rustup target add x86_64-apple-darwin && cargo build --release -p greg-app -p greg-cli --target x86_64-apple-darwin

FROM build AS publish-macos-arm64
RUN rustup target add aarch64-apple-darwin && cargo build --release -p greg-app -p greg-cli --target aarch64-apple-darwin

FROM scratch AS artifact-windows
COPY --from=publish-windows /src/target/x86_64-pc-windows-gnu/release/gregmodmanager.exe /gregmodmanager.exe
COPY --from=publish-windows /src/target/x86_64-pc-windows-gnu/release/gregcli.exe /gregcli.exe

FROM scratch AS artifact-macos-x64
COPY --from=publish-macos-x64 /src/target/x86_64-apple-darwin/release/gregmodmanager /gregmodmanager

FROM scratch AS artifact-macos-arm64
COPY --from=publish-macos-arm64 /src/target/aarch64-apple-darwin/release/gregmodmanager /gregmodmanager
