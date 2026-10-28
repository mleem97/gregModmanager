# Rust (rust + Slint) desktop dev container: rustup toolchain plus the native
# X11/font/GL deps so the Slint UI builds and can at least smoke-run headless
# via xvfb. Release builds cross-compile through the same toolchain.
FROM rust:1-bookworm

ARG USERNAME=vscode
ARG USER_UID=1000
ARG USER_GID=1000

RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        git sudo xvfb \
        libice6 libsm6 libx11-6 libxkbcommon0 libfontconfig1 libfreetype6 \
        libgl1 libegl1 zlib1g dbus-x11 \
    && echo "$USERNAME ALL=(root) NOPASSWD:ALL" > "/etc/sudoers.d/$USERNAME" \
    && chmod 0440 "/etc/sudoers.d/$USERNAME" \
    && rm -rf /var/lib/apt/lists/*

USER "$USERNAME"
WORKDIR /src
COPY . .
RUN rustup component add clippy rustfmt && cargo fetch

RUN rustc --version && cargo --version
