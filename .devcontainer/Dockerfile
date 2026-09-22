# Avalonia desktop (net10 LTS): SDK 10 plus native X11/font/GL deps so the
# Modmanager builds and can at least smoke-run headless via xvfb.
FROM mcr.microsoft.com/dotnet/sdk:10.0-bookworm-slim

ARG USERNAME=vscode
ARG USER_UID=1000
ARG USER_GID=1000

RUN groupadd --gid "$USER_GID" "$USERNAME" \
  && useradd --uid "$USER_UID" --gid "$USER_GID" -m -s /bin/bash "$USERNAME" \
  && apt-get update \
  && apt-get install -y --no-install-recommends \
    git sudo xvfb \
    libice6 libsm6 libx11-6 libxkbcommon0 libfontconfig1 libfreetype6 \
    libgl1 libegl1 zlib1g dbus-x11 \
  && echo "$USERNAME ALL=(root) NOPASSWD:ALL" > "/etc/sudoers.d/$USERNAME" \
  && chmod 0440 "/etc/sudoers.d/$USERNAME" \
  && rm -rf /var/lib/apt/lists/*

USER "$USERNAME"
RUN dotnet --list-sdks
