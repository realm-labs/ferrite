# G01-P6-S014 — PLY-003 Special Player Movement

## Result

Complete. `PLY-MOVEMENT-SPECIAL-001` maps to one modular production owner and one behavioral test
owner. Fluid, swimming, fall-flight/glider, and ability-flight modes share collision and relative
input primitives without changing the ordinary travel default.

## Evidence

Production owner:

- `ferrite-gameplay::player::special_travel`;
- shared primitive extension in `ferrite-gameplay::player::travel`.

Committed test owner:

- `crates/ferrite-gameplay/tests/slices/player/ply_003.rs`.

Validated commands:

```text
cargo test -p ferrite-gameplay --test slices player_ply_003 -- --nocapture
cargo clippy -p ferrite-gameplay --all-targets --all-features -- -D warnings
cargo ferrite task check
git diff --check
```

Focused result before the universal gate:

```text
6 PLY-003 slice tests passed; 0 failed
1 source-specified slice
```

The implementation preserves Water/Lava dispatcher priority, airborne efficiency halving, shallow
Lava's two gravity stages, strict swimming-look boundaries, ordered fall-flight integration,
damage-before-event glider maintenance across every valid slot, and ability flight's post-super
vertical overwrite.
