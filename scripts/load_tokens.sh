#!/usr/bin/env bash
# Load secrets managed by sops + age into the current shell environment.
#
# Usage (in a shell session):
#   source scripts/load_tokens.sh
#
# Decrypted to stdout then evaled -- nothing is written to disk.
# sops finds the age private key in this order:
#   1. $SOPS_AGE_KEY
#   2. $AGE_KEY
#   3. ~/.config/sops/age/keys.txt
set -euo pipefail

if ! command -v sops >/dev/null 2>&1; then
  echo "load_tokens: sops command not found" >&2
  return 1 2>/dev/null || exit 1
fi

SECRETS_FILE="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)/secrets/tokens.env"
if [[ ! -f "$SECRETS_FILE" ]]; then
  echo "load_tokens: secrets file not found: $SECRETS_FILE" >&2
  return 1 2>/dev/null || exit 1
fi

decrypted_secrets="$(sops --decrypt "$SECRETS_FILE")" || {
  echo "load_tokens: failed to decrypt $SECRETS_FILE" >&2
  return 1 2>/dev/null || exit 1
}
eval "$decrypted_secrets"
unset decrypted_secrets
echo "load_tokens: injected secrets from $SECRETS_FILE" >&2
