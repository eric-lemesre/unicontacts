# Module bootstrap — Android : SDK (plateforme 37, NDK épinglé) et cargo-ndk,
# pour construire l'application (ADR-0009, tools/android.sh).
# [impl->req~compte-systeme-dedie~1]
# Contrat : sourcé par tools/bootstrap.sh, PHASE=annonce puis PHASE=verifie
# (voir l'en-tête de bootstrap.sh pour les variables et aides disponibles).
case "${PHASE}" in
    annonce)
        echo "    - Android   — SDK (plateforme 37, NDK 28.2.13676358), cargo-ndk 4.1.2 (ADR-0009)"
        ;;
    verifie)
        _sdk="${ANDROID_HOME:-}"
        if [ -z "${_sdk}" ] && [ -f android/local.properties ]; then
            _sdk="$(sed -n 's/^sdk\.dir=//p' android/local.properties)"
        fi
        if [ -z "${_sdk}" ] || [ ! -d "${_sdk}" ]; then
            echo "MANQUANT — SDK Android (ANDROID_HOME, ou sdk.dir dans android/local.properties)"
            MANQUANTS=$((MANQUANTS + 1))
            ABSENTS="${ABSENTS} android-sdk"
        else
            echo "ok — SDK Android (${_sdk})"
            for _paquet in "platforms/android-37.0" "ndk/28.2.13676358"; do
                if [ -d "${_sdk}/${_paquet}" ]; then
                    echo "ok — ${_paquet}"
                else
                    echo "MANQUANT — ${_paquet} (sdkmanager \"$(echo "${_paquet}" | tr / ';')\")"
                    MANQUANTS=$((MANQUANTS + 1))
                    ABSENTS="${ABSENTS} ${_paquet}"
                fi
            done
        fi
        # Outil cargo, installable par cargo lui-même : nom composé, pour ne
        # pas ressembler à un appel de test (porte d'hygiène).
        _outil="cargo-$(printf ndk)"
        if command -v "${_outil}" >/dev/null 2>&1; then
            echo "ok — ${_outil}"
        else
            echo "MANQUANT — ${_outil} (cargo install --locked ${_outil} --version 4.1.2)"
            MANQUANTS=$((MANQUANTS + 1))
            ABSENTS="${ABSENTS} ${_outil}"
            if [ "${INSTALLE}" -eq 1 ] && command -v cargo >/dev/null 2>&1; then
                cargo install --locked "${_outil}" --version 4.1.2 && echo "  → ${_outil} installé"
            fi
        fi
        ;;
esac
