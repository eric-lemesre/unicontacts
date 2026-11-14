#!/bin/sh
# Construction de l'application Android (ADR-0009).
# [impl->req~compte-systeme-dedie~1]
#
#   sh tools/android.sh natif      # bibliothèque Rust (cargo-ndk) + liaisons Kotlin (UniFFI)
#   sh tools/android.sh verifier   # natif, puis lint, tests JVM et APK de débogage (porte CI)
#
# La bibliothèque native se construit pour arm64-v8a (le Fairphone) et
# x86_64 (l'émulateur), en release, dans android/app/build/rust/jniLibs ;
# les liaisons Kotlin, générées par le uniffi-bindgen du workspace (même
# version qu'UniFFI dans la bibliothèque), dans
# android/app/build/generated/uniffi. Rien n'est versionné de ce qui est
# produit ici (R8). Gradle appelle « natif » avant chaque construction.
#
# Prérequis (tools/bootstrap.d/70-android.sh) : SDK Android (ANDROID_HOME ou
# android/local.properties) avec la plateforme 37 et le NDK épinglé, cibles
# Rust Android (rust-toolchain.toml), cargo-ndk.
#
# POSIX sh strict.
set -eu

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "${ROOT}"

ABIS="arm64-v8a x86_64"
# NDK épinglé (ADR-0009) : installé dans le SDK par
# « sdkmanager "ndk;${NDK_VERSION}" ».
NDK_VERSION="28.2.13676358"
MIN_SDK=33
JNI="${ROOT}/android/app/build/rust/jniLibs"
LIAISONS="${ROOT}/android/app/build/generated/uniffi"

sdk() {
    if [ -n "${ANDROID_HOME:-}" ]; then
        printf '%s' "${ANDROID_HOME}"
    elif [ -f "${ROOT}/android/local.properties" ]; then
        sed -n 's/^sdk\.dir=//p' "${ROOT}/android/local.properties"
    fi
}

ndk() {
    _sdk="$(sdk)"
    [ -n "${_sdk}" ] || return 1
    printf '%s' "${_sdk}/ndk/${NDK_VERSION}"
}

natif() {
    _ndk="$(ndk)" || true
    if [ -z "${_ndk}" ] || [ ! -d "${_ndk}" ]; then
        echo "ÉCHEC : NDK ${NDK_VERSION} introuvable dans le SDK (ANDROID_HOME ou android/local.properties ; sdkmanager \"ndk;${NDK_VERSION}\")" >&2
        return 1
    fi
    if ! command -v cargo-ndk >/dev/null 2>&1; then
        echo "ÉCHEC : cargo-ndk absent (sh tools/bootstrap.sh --verifie)" >&2
        return 1
    fi
    _cibles=""
    for _abi in ${ABIS}; do
        _cibles="${_cibles} -t ${_abi}"
    done
    # shellcheck disable=SC2086 — découpage voulu des options de cible.
    ANDROID_NDK_HOME="${_ndk}" CARGO_INCREMENTAL=0 cargo ndk ${_cibles} \
        --platform "${MIN_SDK}" -o "${JNI}" build --locked --release -p unicontacts-ffi

    # Liaisons générées depuis la bibliothèque arm64 : ses métadonnées
    # UniFFI sont celles de toutes les cibles.
    _lib="${JNI}/arm64-v8a/libunicontacts_ffi.so"
    rm -rf "${LIAISONS}"
    CARGO_INCREMENTAL=0 cargo run --locked --quiet -p unicontacts-ffi --bin uniffi-bindgen -- \
        generate --library "${_lib}" --language kotlin --no-format --out-dir "${LIAISONS}"
}

verifier() {
    natif
    cd "${ROOT}/android"
    ./gradlew --no-daemon --quiet lintDebug testDebugUnitTest assembleDebug
}

case "${1:-}" in
    natif) natif ;;
    verifier) verifier ;;
    *)
        echo "usage : sh tools/android.sh natif|verifier" >&2
        exit 2
        ;;
esac
