# Unicontacts

Unified contact management for Android, with a business core written in Rust.

A single master repository holds the contacts, managed as one block. Sources
(CardDAV, Google, SIM, local storage…) are only synchronisation targets; any
non-trivial divergence is submitted to the user.

The project is instantiated from the
[alm-forge](https://github.com/eric-lemesre/alm-forge) ALM template: traced
requirements from need to mechanized tests, ADRs, quality gates enforced
locally and in CI. Normative documentation is in French: see README.md,
docs/SPEC.md and docs/methode/.

Getting started: `just setup`, then `just ci` before proposing a change.

Licence: [GPL-3.0-or-later](LICENSE).
