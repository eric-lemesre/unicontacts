#!/bin/sh
# Tests d'intégration du connecteur CardDAV contre Radicale (ADR-0008).
# [itest->req~source-carddav~1]
#
#   sh tools/radicale.sh installer   # environnement Python épinglé, dans .cache/
#   sh tools/radicale.sh test        # installe au besoin, démarre, teste, arrête
#
# Radicale tourne sur 127.0.0.1, sur un port libre, avec un stockage jetable
# (mktemp) retiré à la sortie. Les tests d'intégration sont marqués #[ignore]
# dans crates/carddav/tests/ : un simple « cargo test » les montre ignorés ;
# ce script les lance avec --ignored, l'URL du serveur dans
# UNICONTACTS_RADICALE.
#
# Échoue — jamais ne passe sans rien tester (R3) — si Python ou son module
# venv manque, si l'installation épinglée échoue, si Radicale ne répond pas
# dans le délai, ou si un test échoue.
#
# POSIX sh strict.
set -eu

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "${ROOT}"

VERSION="3.9.1"
ENV="${ROOT}/.cache/radicale-${VERSION}"
EXIGENCES="${ROOT}/tools/radicale-requirements.txt"

installer() {
    if [ -x "${ENV}/bin/radicale" ]; then
        return 0
    fi
    if ! command -v python3 >/dev/null 2>&1; then
        echo "ÉCHEC : python3 introuvable (sh tools/bootstrap.sh --verifie)" >&2
        return 1
    fi
    echo "Installation de Radicale ${VERSION} dans ${ENV}…" >&2
    rm -rf "${ENV}"
    mkdir -p "${ROOT}/.cache"
    if ! python3 -m venv "${ENV}"; then
        echo "ÉCHEC : python3 -m venv a échoué (paquet python3-venv absent ?)" >&2
        rm -rf "${ENV}"
        return 1
    fi
    if ! "${ENV}/bin/python" -m pip install --quiet --disable-pip-version-check \
            --no-deps --require-hashes -r "${EXIGENCES}"; then
        echo "ÉCHEC : installation épinglée de Radicale refusée" >&2
        rm -rf "${ENV}"
        return 1
    fi
}

port_libre() {
    "${ENV}/bin/python" -c 'import socket; s = socket.socket(); s.bind(("127.0.0.1", 0)); print(s.getsockname()[1]); s.close()'
}

tester() {
    installer
    stockage="$(mktemp -d)"
    journal="${stockage}.journal"
    port="$(port_libre)"
    "${ENV}/bin/radicale" \
        --storage-filesystem-folder "${stockage}/collections" \
        --server-hosts "127.0.0.1:${port}" \
        --auth-type none --rights-type owner_only \
        --logging-level warning >"${journal}" 2>&1 &
    pid=$!
    # shellcheck disable=SC2064 — valeurs figées à la pose du piège, voulu.
    trap "kill ${pid} 2>/dev/null || true; rm -rf '${stockage}' '${journal}'" EXIT INT TERM

    url="http://127.0.0.1:${port}"
    essais=0
    until curl -s -o /dev/null "${url}/"; do
        essais=$((essais + 1))
        if [ "${essais}" -ge 50 ] || ! kill -0 "${pid}" 2>/dev/null; then
            echo "ÉCHEC : Radicale ne répond pas sur ${url}" >&2
            cat "${journal}" >&2
            return 1
        fi
        sleep 0.2
    done
    echo "Radicale ${VERSION} sur ${url}" >&2

    UNICONTACTS_RADICALE="${url}" CARGO_INCREMENTAL=0 \
        cargo test --locked -p unicontacts-carddav --test radicale -- --ignored
}

case "${1:-}" in
    installer) installer ;;
    test) tester ;;
    *)
        echo "usage : sh tools/radicale.sh installer|test" >&2
        exit 2
        ;;
esac
