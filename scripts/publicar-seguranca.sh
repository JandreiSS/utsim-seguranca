#!/usr/bin/env bash
# Publica o UTSim Segurança pelo launcher, como produto próprio (`seguranca`): monta a
# pasta do programa, cifra, assina e grava o pacote e o manifesto na pasta de publicação (local). Não envia nada à internet: depois, 'utsim-publicar github' sobe o pacote e o
# manifesto para o GitHub. O launcher já instalado passa a mostrar o produto; ele não precisa mudar.
#
# Uso: bash scripts/publicar-seguranca.sh <pasta_de_publicação> [canal] [versão]
#   canal   padrão: estavel (o launcher 0.4.0 mostra todos; o canal só conta para launchers antigos)
#   versão  padrão: a de crates/utsim-seguranca-app/Cargo.toml (versão própria, tag seg-v<versão>)
#
# Antes: cargo build --release -p utsim-seguranca-app -p utsim-publicar
# Notas da versão: a seção "## [seg-<versão>]" do CHANGELOG.md, se existir; senão, os itens
# da seção "## [Não lançado]" que citam o UTSim Segurança.

set -eu

cd "$(dirname "$0")/.." || exit 1

[ $# -ge 1 ] || { echo "uso: $0 <pasta_de_publicação> [canal] [versão]" >&2; exit 2; }
SAIDA=$1
CANAL=${2:-estavel}
VERSAO=${3:-$(awk -F'"' '/^version = /{print $2; exit}' crates/utsim-seguranca-app/Cargo.toml)}
VERSAO=${VERSAO#seg-}
VERSAO=${VERSAO#v}

PUBLICAR=target/release/utsim-publicar
[ -x "$PUBLICAR" ] || PUBLICAR=target/release/utsim-publicar.exe
[ -x "$PUBLICAR" ] || { echo "erro: rode antes cargo build --release -p utsim-publicar" >&2; exit 1; }

case "$(uname -s)" in
    MINGW*|MSYS*|CYGWIN*) PLATAFORMA=windows-x64; NOME=utsim-seguranca.exe ;;
    Linux) PLATAFORMA=linux-x64; NOME=utsim-seguranca ;;
    *) echo "erro: plataforma não suportada" >&2; exit 1 ;;
esac
BIN=target/release/$NOME
[ -f "$BIN" ] || { echo "erro: $BIN não encontrado; rode antes cargo build --release -p utsim-seguranca-app" >&2; exit 1; }

PASTA=$(mktemp -d)
trap 'rm -rf "$PASTA"' EXIT
mkdir -p "$PASTA/seguranca/bin" "$PASTA/seguranca/manual"
cp "$BIN" "$PASTA/seguranca/bin/"
cp docs/manual/src/seguranca-app.md "$PASTA/seguranca/manual/"

if grep -q "^## \[seg-$VERSAO\]" CHANGELOG.md; then
    bash scripts/notas-versao.sh "seg-$VERSAO" > "$PASTA/notas.md"
else
    awk '/^## \[/ { n++ } n == 1 && /^- / && /UTSim Segurança/ { print }' CHANGELOG.md > "$PASTA/notas.md"
fi
[ -s "$PASTA/notas.md" ] || echo "(sem notas)" > "$PASTA/notas.md"

"$PUBLICAR" publicar \
    --saida "$SAIDA" \
    --produto seguranca \
    --nome "UTSim Segurança" \
    --descricao "Segurança do trabalho da família UTSim" \
    --subtitulo "Segurança do trabalho" \
    --autor "UTSim" \
    --cor "#F28C1E" \
    --icone assets/icone/familia/seguranca_256.png \
    --versao "$VERSAO" \
    --canal "$CANAL" \
    --plataforma "$PLATAFORMA" \
    --pasta "$PASTA/seguranca" \
    --executavel "bin/$NOME" \
    --notas "$PASTA/notas.md"
