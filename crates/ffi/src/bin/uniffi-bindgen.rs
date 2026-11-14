//! Générateur des liaisons Kotlin, embarqué dans le workspace pour garder
//! la version d'UniFFI de la bibliothèque (ADR-0009) :
//! `sh tools/android.sh natif` l'appelle.
// [impl->req~synchronisation-directe~1]

fn main() {
    uniffi::uniffi_bindgen_main();
}
