# Frozen approved practice reference

These three files are the untouched gameplay-core source from the preserved
pre-multiplayer snapshot `/private/tmp/burnhop-mp-baseline`, created during
milestone 4 on 2026-09-11. Their SHA-256 values also match the earlier
`burnhop-combat-baseline.json` ledger. They are checked-in test fixtures, not
runtime code or a Cargo dependency. Do not update them to match current output.

| File | SHA-256 |
| --- | --- |
| `lib.rs` | `0f6cd62ace1e02e39072769c10155a78e52766261a6d5cef1156fe08527cec30` |
| `combat.rs` | `b496155cbc89f75731143a789a946d34955510d2acea9840ec597346e11aa4e4` |
| `collision.rs` | `f8237bdbd6f3083b34fbfabe14934824d7756249736baa33ce7671bb84f709ea` |

`tests/practice_regression.rs` compiles the original practice implementation and
runs the same 12,000 commands against it and the current multiplayer adapter.
Every state/event Debug field is compared at every tick without float rounding,
tolerances or an accepted Windows hash copied from the implementation. Only the
historical actor-label rename is normalized. The original Apple Silicon Mac
hash assertion is retained on that target as an additional fixture check.

The first Windows CI run showed that its full-precision trace differs from the
Mac golden. Both versions use [`f64::hypot`](https://doc.rust-lang.org/std/primitive.f64.html#method.hypot),
whose precision is platform dependent. Comparing independent implementations on
the same platform tests practice equivalence without claiming cross-platform
bitwise determinism. A mismatch prints the first differing tick and full states.
