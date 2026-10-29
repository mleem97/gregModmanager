// Jenkins declarative pipeline: Rust check / build matrix.
//
// Runs the same quality gates as the local gate (`scripts/check.sh`) and the
// Rust CI jobs in `.forgejo/workflows/rust.yml` (fmt, clippy, test) plus a
// workspace build, across Linux, Windows and macOS agents.
//
// Prerequisites on the Jenkins agents:
//   - Linux:   Rust via rustup (bootstrap in stage) + sudo apt for Slint system
//              dependencies (libxcb-render0-dev, libxcb-shape0-dev,
//              libxcb-xfixes0-dev, libxkbcommon-dev, libfontconfig1-dev), or a
//              preinstalled toolchain.
//   - Windows: Rust via rustup (`rustup component add rustfmt clippy`).
//   - macOS:   Rust via rustup (`rustup component add rustfmt clippy`).

pipeline {
    agent { label "${matrix.os}" }
    name "Rust CI / ${matrix.os}"
    timeout { time: 45, unit: 'MINUTES' }
    environment {
        CARGO_TERM_COLOR = 'always'
    }
    stages {
        stage('Checkout') {
            steps {
                checkout scm
            }
        }
        stage('Rust-Toolchain') {
            steps {
                script {
                    if (matrix.os == 'Windows') {
                        bat 'rustup component add rustfmt clippy'
                    } else if (matrix.os == 'Linux') {
                        sh '''
                            set -e
                            curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal --component rustfmt,clippy
                            . "$HOME/.cargo/env"
                            rustup component add rustfmt clippy
                            sudo apt-get update
                            sudo apt-get install -y --no-install-recommends \
                                libxcb-render0-dev \
                                libxcb-shape0-dev \
                                libxcb-xfixes0-dev \
                                libxkbcommon-dev \
                                libfontconfig1-dev
                        '''
                    } else {
                        sh 'rustup component add rustfmt clippy'
                    }
                }
            }
        }
        stage('Format') {
            steps {
                script {
                    if (matrix.os == 'Windows') {
                        bat 'cargo fmt --all -- --check'
                    } else {
                        sh 'cargo fmt --all -- --check'
                    }
                }
            }
        }
        stage('Clippy') {
            steps {
                script {
                    if (matrix.os == 'Windows') {
                        bat 'cargo clippy --workspace --all-targets -- -D warnings'
                    } else {
                        sh 'cargo clippy --workspace --all-targets -- -D warnings'
                    }
                }
            }
        }
        stage('Test') {
            steps {
                script {
                    if (matrix.os == 'Windows') {
                        bat 'cargo test --workspace'
                    } else {
                        sh 'cargo test --workspace'
                    }
                }
            }
        }
        stage('Build') {
            steps {
                script {
                    if (matrix.os == 'Windows') {
                        bat 'cargo build --workspace'
                    } else {
                        sh 'cargo build --workspace'
                    }
                }
            }
        }
    }
}

matrix {
    axes {
        axis {
            name 'os'
            values 'linux', 'windows', 'macos'
        }
    }
}
