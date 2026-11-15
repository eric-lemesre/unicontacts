#!/bin/sh
# Calendrier des fêtes du prénom, tiré de Wikidata (ADR-0013).
#
#   sh tools/fetes.sh regenerer   # interroge Wikidata, réécrit le calendrier
#   sh tools/fetes.sh verifier    # contrôle l'empreinte du calendrier versionné
#
# Source : propriété P1750 (« fête du prénom ») de Wikidata, qualifiée par
# P17 = France (Q142) : le calendrier traditionnel français. Licence CC0.
# Le calendrier versionné (crates/core/data/fetes-fr.txt) ne s'édite jamais
# à la main (R8) : on le régénère, on relit le diff, on committe le fichier
# et son empreinte (crates/core/data/fetes-fr.txt.sha256).
#
# La régénération demande le réseau ; la vérification, non : elle tourne en
# voie documentaire.
#
# POSIX sh, curl, awk, sort, sha256sum.
# [impl->req~fete-du-prenom-proposee~1]
set -eu

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
CALENDRIER="${ROOT}/crates/core/data/fetes-fr.txt"
EMPREINTE="${CALENDRIER}.sha256"
POINT="https://query.wikidata.org/sparql"

REQUETE='SELECT ?nomLabel ?jourLabel WHERE {
  ?nom p:P1750 ?declaration .
  ?declaration ps:P1750 ?jour ; pq:P17 wd:Q142 .
  ?nom rdfs:label ?nomLabel . FILTER(LANG(?nomLabel) = "fr")
  ?jour rdfs:label ?jourLabel . FILTER(LANG(?jourLabel) = "fr")
}'

regenerer() {
    brut="$(mktemp)"
    trap 'rm -f "${brut}"' EXIT
    curl --fail --silent --show-error --get \
        --user-agent "unicontacts-fetes/1.0 (outil de generation)" \
        --header "Accept: text/tab-separated-values" \
        --data-urlencode "query=${REQUETE}" \
        "${POINT}" > "${brut}"
    {
        echo "# Fêtes du prénom, calendrier traditionnel français."
        echo "# Source : Wikidata, P1750 qualifiée par P17 = Q142 (CC0)."
        echo "# Généré par « sh tools/fetes.sh regenerer » le $(date -u +%Y-%m-%d) :"
        echo "# ne pas éditer à la main (R8, ADR-0013)."
        echo "# Format : MM-JJ, tabulation, prénom."
        # Lignes « "Prénom"@fr<TAB>"18 mai"@fr » ; « 1er » pour le premier.
        awk -F '\t' 'BEGIN {
            n = split("janvier février mars avril mai juin juillet août septembre octobre novembre décembre", noms, " ")
            for (i = 1; i <= n; i++) mois[noms[i]] = i
        }
        NR > 1 {
            nom = $1; jour = $2
            gsub(/^"|"@fr$/, "", nom); gsub(/^"|"@fr$/, "", jour)
            split(jour, p, " ")
            j = p[1]; sub(/er$/, "", j)
            if (nom == "" || j !~ /^[0-9]+$/ || !(p[2] in mois)) next
            printf "%02d-%02d\t%s\n", mois[p[2]], j, nom
        }' "${brut}" | LC_ALL=C sort -u
    } > "${CALENDRIER}"
    sha256sum "${CALENDRIER}" | awk '{ print $1 }' > "${EMPREINTE}"
    echo "Calendrier régénéré : $(grep -vc '^#' "${CALENDRIER}") fête(s)."
}

verifier() {
    attendue="$(cat "${EMPREINTE}")"
    obtenue="$(sha256sum "${CALENDRIER}" | awk '{ print $1 }')"
    if [ "${attendue}" != "${obtenue}" ]; then
        echo "ÉCHEC : calendrier des fêtes modifié hors de « sh tools/fetes.sh regenerer »" >&2
        exit 1
    fi
    echo "Calendrier des fêtes : OK (empreinte conforme)."
}

case "${1:-}" in
    regenerer) regenerer ;;
    verifier) verifier ;;
    *)
        echo "usage : sh tools/fetes.sh regenerer|verifier" >&2
        exit 2
        ;;
esac
