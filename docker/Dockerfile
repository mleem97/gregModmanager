# syntax=docker/dockerfile:1.7
# Linux SDK container used for deterministic build/test and cross-RID publish.
FROM mcr.microsoft.com/dotnet/runtime:9.0-noble AS runtime9

FROM mcr.microsoft.com/dotnet/sdk:10.0-noble AS restore
WORKDIR /src

# The solution is built by SDK 10 (global.json) but testhost targets net9.0.
COPY --from=runtime9 /usr/share/dotnet/shared/Microsoft.NETCore.App/ /usr/share/dotnet/shared/Microsoft.NETCore.App/

COPY global.json GregModmanager.sln ./
COPY src/GregModmanager.Avalonia/GregModmanager.Avalonia.csproj src/GregModmanager.Avalonia/
COPY src/GregModmanager.Core/GregModmanager.Core.csproj src/GregModmanager.Core/
COPY tests/GregModmanager.Tests/GregModmanager.Tests.csproj tests/GregModmanager.Tests/

RUN --mount=type=cache,id=nuget,target=/root/.nuget/packages \
    dotnet restore GregModmanager.sln

FROM restore AS build
COPY . ./
RUN --mount=type=cache,id=nuget,target=/root/.nuget/packages \
    dotnet build GregModmanager.sln --no-restore --configuration Release

FROM build AS test
RUN --mount=type=cache,id=nuget,target=/root/.nuget/packages \
    dotnet test GregModmanager.sln --no-build --configuration Release --verbosity normal

# Cross-publish only: these Linux containers cannot execute Windows or macOS UI binaries.
FROM build AS publish-windows
RUN --mount=type=cache,id=nuget,target=/root/.nuget/packages \
    dotnet publish src/GregModmanager.Avalonia/GregModmanager.Avalonia.csproj \
      --configuration Release --runtime win-x64 --self-contained true \
      --output /out/win-x64

FROM scratch AS artifact-windows
COPY --from=publish-windows /out/ /

FROM build AS publish-macos-x64
RUN --mount=type=cache,id=nuget,target=/root/.nuget/packages \
    dotnet publish src/GregModmanager.Avalonia/GregModmanager.Avalonia.csproj \
      --configuration Release --runtime osx-x64 --self-contained true \
      --output /out/osx-x64

FROM scratch AS artifact-macos-x64
COPY --from=publish-macos-x64 /out/ /

FROM build AS publish-macos-arm64
RUN --mount=type=cache,id=nuget,target=/root/.nuget/packages \
    dotnet publish src/GregModmanager.Avalonia/GregModmanager.Avalonia.csproj \
      --configuration Release --runtime osx-arm64 --self-contained true \
      --output /out/osx-arm64

FROM scratch AS artifact-macos-arm64
COPY --from=publish-macos-arm64 /out/ /
