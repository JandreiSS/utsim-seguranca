#!/usr/bin/env bash
# Verificação completa deste repositório: formatação, família, lints, testes e build de release.
# Para no primeiro passo que falhar e termina com "Tudo certo".
#
# Na máquina do autor, pega a trava de compilação (utsim-trava) se quem chamou ainda não a tem.
# Em outra máquina (a de um aluno), roda sem trava.

set -u
cd "$(dirname "$0")/.." || exit 1
REPO=$(basename "$(git rev-parse --show-toplevel 2>/dev/null || pwd)")

TRAVA=/c/Users/Jandrei/AppData/Local/utsim-trava/trava.sh
TRAVA_DONO=/c/Users/Jandrei/AppData/Local/Temp/utsim-verificar.lock/dono
# Quem chamou já tem a trava quando o token dele (exportado por trava.sh) é o dono dela.
if [ -f "$TRAVA" ] && [ "$(cat "$TRAVA_DONO" 2>/dev/null)" != "${UTSIM_TRAVA_TOKEN:-sem-token}" ]; then
    # shellcheck disable=SC1090
    source "$TRAVA" "$REPO:verificar"
fi

export CARGO_INCREMENTAL=0
if command -v sccache >/dev/null 2>&1; then
    export RUSTC_WRAPPER=sccache
    export SCCACHE_CACHE_SIZE="${SCCACHE_CACHE_SIZE:-25G}"
fi

falha() { echo "FALHOU: $1" >&2; exit 1; }
passo() { echo; echo "==> $1"; }

passo "1/5 Formatação (cargo fmt --check)"
cargo fmt --all --check || falha "formatação (rode cargo fmt --all)"

passo "2/5 Família (uma tag só, sem a família local ligada, sem dependência duplicada)"
if [ -f .cargo/config.toml ] && grep -q "utsim-familia" .cargo/config.toml; then
    falha "a família local está ligada (bash scripts/familia-local.sh desliga)"
fi
TAGS=$(grep -ho 'utsim-familia", tag = "[^"]*"' Cargo.toml | sort -u | wc -l)
[ "$TAGS" -le 1 ] || falha "o Cargo.toml usa mais de uma tag da família"
DUP=$(cargo tree -d -e normal --workspace 2>/dev/null | grep -E '^(utsim-[a-z-]+|egui|eframe) v' || true)
[ -z "$DUP" ] || falha "dependência em duas versões (tipos não batem com a família):
$DUP"

passo "3/5 Lints (cargo clippy -D warnings)"
cargo clippy --workspace --all-targets -- -D warnings || falha "clippy"

passo "4/5 Testes (cargo test)"
cargo test --workspace || falha "testes"

passo "5/5 Build de release"
cargo build --workspace --release || falha "build de release"

echo
echo "Tudo certo"
