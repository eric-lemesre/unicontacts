# Module bootstrap — Python 3 et son module venv : environnement épinglé de
# Radicale, serveur CardDAV des tests d'intégration (ADR-0008).
# [itest->req~source-carddav~1]
# Contrat : sourcé par tools/bootstrap.sh, PHASE=annonce puis PHASE=verifie
# (voir l'en-tête de bootstrap.sh pour les variables et aides disponibles).
case "${PHASE}" in
    annonce)
        echo "    - Python 3 (venv) — Radicale épinglé, tests d'intégration CardDAV (ADR-0008)"
        ;;
    verifie)
        verifier "Python 3 (tests d'intégration CardDAV)" python3 \
            "python3-venv" "python3" "python" "Python.Python.3.13" \
            "requis par tools/radicale.sh"
        # Debian et Ubuntu livrent python3 sans ensurepip : venv échoue
        # alors à créer l'environnement. On le dit ici plutôt qu'au premier
        # test d'intégration.
        if command -v python3 >/dev/null 2>&1 \
                && ! python3 -c 'import ensurepip, venv' >/dev/null 2>&1; then
            echo "MANQUANT — module venv de Python (paquet python3-venv)"
            MANQUANTS=$((MANQUANTS + 1))
            ABSENTS="${ABSENTS} python3-venv"
        fi
        ;;
esac
