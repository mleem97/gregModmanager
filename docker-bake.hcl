group "default" {
  targets = ["test"]
}

target "test" {
  context    = "."
  dockerfile = "docker/Dockerfile"
  target     = "test"
}

target "windows-cross-publish" {
  context    = "."
  dockerfile = "docker/Dockerfile"
  target     = "artifact-windows"
  output     = ["type=local,dest=artifacts/docker/windows-cross"]
}

target "macos-x64-publish" {
  context    = "."
  dockerfile = "docker/Dockerfile"
  target     = "artifact-macos-x64"
  output     = ["type=local,dest=artifacts/docker/macos-x64"]
}

target "macos-arm64-publish" {
  context    = "."
  dockerfile = "docker/Dockerfile"
  target     = "artifact-macos-arm64"
  output     = ["type=local,dest=artifacts/docker/macos-arm64"]
}
