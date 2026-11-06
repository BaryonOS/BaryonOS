#!/usr/bin/env bash
# Installeert de Copilot-runner als aparte gebruiker in de bestaande WSL-Ubuntu.
# Geen isolatie: deze gebruiker heeft sudo zonder wachtwoord, dus root in deze distro
# (ADR-CI-COPILOT-WSL-20261106). Gebruik:
#   sudo bash tools/copilot-runner/install-wsl.sh "$(gh api -X POST repos/simonvanieperen/nieuw-os/actions/runners/registration-token --jq .token)"
set -euo pipefail

TOKEN="${1:?registratietoken ontbreekt (1 uur geldig, zie kop van dit script)}"
[ "$(id -u)" = 0 ] || { echo "draai dit script met sudo" >&2; exit 1; }

RUNNER_USER=copilot-runner
VERSION=2.338.2
SHA256=baebf204307bac6193480852732f0aba2ae97b6c5597c244f4d5558390147c40
REPO_URL=https://github.com/simonvanieperen/nieuw-os
LABEL=copilot-wsl
HOME_DIR=/home/$RUNNER_USER
RUNNER_DIR=$HOME_DIR/actions-runner

apt-get update -qq
apt-get install -y -qq qemu-system-x86 qemu-utils ovmf swtpm swtpm-tools \
  build-essential pkg-config git curl jq unzip python3 python3-venv

id "$RUNNER_USER" >/dev/null 2>&1 || useradd -m -s /bin/bash "$RUNNER_USER"
usermod -aG kvm "$RUNNER_USER"
echo "$RUNNER_USER ALL=(ALL) NOPASSWD:ALL" > "/etc/sudoers.d/$RUNNER_USER"
chmod 0440 "/etc/sudoers.d/$RUNNER_USER"
visudo -cf "/etc/sudoers.d/$RUNNER_USER"

# Rust volgens rust-toolchain.toml van de repo.
sudo -u "$RUNNER_USER" -H bash -c '
  set -euo pipefail
  [ -x ~/.cargo/bin/rustup ] || curl --proto "=https" --tlsv1.2 -sSf https://sh.rustup.rs |
    sh -s -- -y --profile minimal --default-toolchain none
  ~/.cargo/bin/rustup toolchain install 1.90.0 --profile minimal \
    -c rustfmt,clippy -t x86_64-unknown-uefi,x86_64-unknown-none'

if [ ! -f "$RUNNER_DIR/.runner" ]; then
  TARBALL=$(mktemp)
  curl -fsSL -o "$TARBALL" \
    "https://github.com/actions/runner/releases/download/v$VERSION/actions-runner-linux-x64-$VERSION.tar.gz"
  echo "$SHA256  $TARBALL" | sha256sum -c -
  install -d -o "$RUNNER_USER" -g "$RUNNER_USER" "$RUNNER_DIR"
  sudo -u "$RUNNER_USER" tar xzf "$TARBALL" -C "$RUNNER_DIR"
  rm -f "$TARBALL"
  "$RUNNER_DIR/bin/installdependencies.sh"
  sudo -u "$RUNNER_USER" -H bash -c "cd '$RUNNER_DIR' && ./config.sh --unattended --replace \
    --url '$REPO_URL' --token '$TOKEN' --name '$(hostname)-copilot' --labels '$LABEL'"
  (cd "$RUNNER_DIR" && ./svc.sh install "$RUNNER_USER")
fi
(cd "$RUNNER_DIR" && ./svc.sh start && ./svc.sh status | head -5)
