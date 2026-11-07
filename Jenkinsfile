// Jenkins declarative pipeline: Rust checks and builds on native agents.
// Required labels: linux, windows, macos. Windows/macOS agents need Rust via
// rustup on PATH; Linux bootstraps rustup when absent and needs passwordless
// sudo for the listed Slint system packages (or install them on the agent).
// Quality commands match scripts/check.sh and cargo xtask check.

def rustCommand(String command) {
    if (env.PLATFORM == 'windows') {
        bat command
    } else {
        // Each sh step starts a fresh shell; retain a newly bootstrapped Rust
        // installation across stages without depending on an interactive shell.
        sh(script: '''
            set -eu
            if [ -f "$HOME/.cargo/env" ]; then
                . "$HOME/.cargo/env"
            fi
        ''' + command)
    }
}

pipeline {
    agent none
    options {
        timeout(time: 120, unit: 'MINUTES')
        skipDefaultCheckout(true)
    }
    parameters {
        booleanParam(name: 'BUILD_TEST_PACKAGES', defaultValue: false, description: 'Build and archive Windows and Linux packages without publishing a release')
        booleanParam(name: 'PUBLISH_RELEASE', defaultValue: false, description: 'Build and publish a GitHub release from main after all gates pass')
        booleanParam(name: 'RELEASE_VALIDATED', defaultValue: false, description: 'Installation, upgrades and runtime behavior were validated on all supported platforms and package formats')
    }
    environment {
        CARGO_TERM_COLOR = 'always'
    }
    stages {
        stage('Build and test') {
            matrix {
                axes {
                    axis {
                        name 'PLATFORM'
                        values 'linux', 'windows', 'macos'
                    }
                }
                agent { label "${PLATFORM}" }
                stages {
                    stage('Checkout') {
                        steps {
                            checkout scm
                        }
                    }
                    stage('Rust toolchain') {
                        steps {
                            script {
                                if (env.PLATFORM == 'linux') {
                                    sh '''
                                        set -eu
                                        if [ -f "$HOME/.cargo/env" ]; then
                                            . "$HOME/.cargo/env"
                                        fi
                                        if ! command -v rustup >/dev/null 2>&1; then
                                            curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal --component rustfmt,clippy
                                            . "$HOME/.cargo/env"
                                        fi
                                        rustup component add rustfmt clippy
                                        if ! command -v cc >/dev/null 2>&1 || ! command -v pkg-config >/dev/null 2>&1 || ! pkg-config --exists xcb-render xcb-shape xcb-xfixes xkbcommon fontconfig; then
                                            # Debian/Ubuntu bootstrap; other Linux agents
                                            # should preinstall the native prerequisites.
                                            sudo -n apt-get update
                                            sudo -n apt-get install -y --no-install-recommends \
                                                build-essential \
                                                pkg-config \
                                                libxcb-render0-dev \
                                                libxcb-shape0-dev \
                                                libxcb-xfixes0-dev \
                                                libxkbcommon-dev \
                                                libfontconfig1-dev
                                        fi
                                    '''
                                } else {
                                    rustCommand('rustup component add rustfmt clippy')
                                }
                            }
                        }
                    }
                    stage('Format') {
                        steps {
                            script {
                                rustCommand('cargo fmt --all -- --check')
                            }
                        }
                    }
                    stage('Clippy') {
                        steps {
                            script {
                                rustCommand('cargo clippy --workspace --all-targets -- -D warnings')
                            }
                        }
                    }
                    stage('Test') {
                        steps {
                            script {
                                rustCommand('cargo test --workspace')
                            }
                        }
                    }
                    stage('Build') {
                        steps {
                            script {
                                rustCommand('cargo build --workspace')
                            }
                        }
                    }
                    stage('Release packages') {
                        when {
                            expression { (params.BUILD_TEST_PACKAGES || (params.PUBLISH_RELEASE && env.BRANCH_NAME == 'main')) && env.PLATFORM != 'macos' }
                        }
                        steps {
                            script {
                                if (env.PLATFORM == 'linux') {
                                    sh '''
                                        set -eu
                                        [ ! -f "$HOME/.cargo/env" ] || . "$HOME/.cargo/env"
                                        version=$(tr -d '\\r\\n' < VERSION)
                                        rm -rf dist
                                        cargo xtask dist
                                        max_glibc=$(objdump -T "dist/gregmodmanager-${version}-linux-x64/gregmodmanager" | grep -oE 'GLIBC_[0-9]+([.][0-9]+)+' | sed 's/GLIBC_//' | sort -Vu | tail -n 1)
                                        [ -n "$max_glibc" ] || { echo 'Could not determine glibc baseline' >&2; exit 1; }
                                        echo "Linux package requires glibc $max_glibc"
                                        if [ "$PUBLISH_RELEASE" = true ] && [ "$(printf '%s\\n%s\\n' 2.39 "$max_glibc" | sort -V | tail -n 1)" != 2.39 ]; then
                                            echo 'Release requires an Ubuntu 24.04-compatible glibc baseline (2.39 or earlier)' >&2
                                            exit 1
                                        fi
                                        build/scripts/linux/build-rust-packages.sh "dist/gregmodmanager-${version}-linux-x64" "$version" false dist/packages
                                        cp dist/packages/*.deb dist/packages/*.rpm dist/packages/*.apk dist/packages/*.pkg.tar.zst dist/
                                        for file in dist/*.deb dist/*.rpm dist/*.apk dist/*.pkg.tar.zst; do
                                            (cd dist && sha256sum "$(basename "$file")" > "$(basename "$file").sha256")
                                        done
                                        git rev-parse HEAD > dist/linux-commit.txt
                                    '''
                                    stash name: 'linux-release', includes: 'dist/*.tar.gz,dist/*.deb,dist/*.rpm,dist/*.apk,dist/*.pkg.tar.zst,dist/*.sha256,dist/linux-commit.txt'
                                    archiveArtifacts artifacts: 'dist/*.tar.gz,dist/*.deb,dist/*.rpm,dist/*.apk,dist/*.pkg.tar.zst,dist/*.sha256', fingerprint: true
                                } else {
                                    powershell "if (Test-Path dist) { Remove-Item -Recurse -Force dist }"
                                    bat 'cargo xtask dist'
                                    powershell '''
                                        $ErrorActionPreference = 'Stop'
                                        $version = (Get-Content VERSION -Raw).Trim()
                                        & build/installer/build-windows.ps1 -StageDir "dist/gregmodmanager-$version-win-x64" -Version $version -OutputDir dist
                                        if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
                                        git rev-parse HEAD | Set-Content -Encoding ascii dist/windows-commit.txt
                                    '''
                                    stash name: 'windows-release', includes: 'dist/*.zip,dist/*.exe,dist/*.msi,dist/*.sha256,dist/windows-commit.txt'
                                    archiveArtifacts artifacts: 'dist/*.zip,dist/*.exe,dist/*.msi,dist/*.sha256', fingerprint: true
                                }
                            }
                        }
                    }
                }
            }
        }
        stage('Publish GitHub release') {
            when { expression { params.PUBLISH_RELEASE && env.BRANCH_NAME == 'main' } }
            agent { label 'linux' }
            steps {
                script {
                    if (!params.RELEASE_VALIDATED) {
                        error('Release validation must be completed for every supported package before publication.')
                    }
                    error('Release blocked: the Alpine APK is known not to run. Fix and validate it before publication.')
                }
                checkout scm
                sh 'rm -rf dist'
                unstash 'linux-release'
                unstash 'windows-release'
                withCredentials([string(credentialsId: 'github-release-token', variable: 'GH_TOKEN')]) {
                    sh '''
                        set -eu
                        version=$(tr -d '\\r\\n' < VERSION)
                        tag="v$version"
                        repo=mleem97/gregModmanager
                        [ "$(tr -d '\\r\\n' < dist/linux-commit.txt)" = "$GIT_COMMIT" ]
                        [ "$(tr -d '\\r\\n' < dist/windows-commit.txt)" = "$GIT_COMMIT" ]
                        # Fail closed on network errors and existing tags. Never overwrite a release.
                        set +e
                        git ls-remote --exit-code --tags "https://github.com/$repo.git" "refs/tags/$tag" >/dev/null
                        result=$?
                        set -e
                        [ "$result" -eq 2 ] || { echo "Tag already exists or GitHub is unavailable (status $result)" >&2; exit 1; }
                        gh api "repos/$repo/commits/$GIT_COMMIT" --silent
                        gh release create "$tag" dist/*.zip dist/*.tar.gz dist/*.exe dist/*.msi dist/*.deb dist/*.rpm dist/*.apk dist/*.pkg.tar.zst dist/*.sha256 --repo "$repo" --target "$GIT_COMMIT" --title "$tag" --notes "See CHANGELOG.md for release notes."
                    '''
                }
            }
        }
    }
}
