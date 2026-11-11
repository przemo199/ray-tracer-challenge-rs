# AGENTS.md

Guidance for AI coding agents (and humans) working in this repository.
`CLAUDE.md` imports this file; keep tool-agnostic instructions here.

## Project

A Rust implementation of the ray tracer from *The Ray Tracer Challenge* (Jamis Buck).
Cargo workspace, edition 2024, stable toolchain (`rust-toolchain.toml`).

| Crate | Kind | Purpose |
|---|---|---|
| `ray-tracer/` | lib (`ray_tracer`) | All rendering math and logic. No I/O except `Canvas::to_png_file`. |
| `ray-tracer-cli/` | bin (`ray-tracer-cli`) | Parses a YAML scene file, renders it, writes a PNG. |

Other top-level directories:
- `scenes/*.yaml` – example scenes (the book's YAML scene format).
- `rendered_images/*.png` – images embedded in `README.md`. **Do not overwrite** unless asked.
- `target/` – build output (ignored). Put any scratch renders in `target/renders/`.

### Library layout (`ray-tracer/src`)

Each module is a `foo.rs` file that re-exports from a `foo/` directory (no `mod.rs`).

- `primitives/` – `Point`, `Vector`, `Color`, `Matrix<N>` (const generic), `Transformation` (= `Matrix<4>`),
  `transformations::{translation, scaling, rotation_x/y/z, shearing, view_transform}`, `Light`, `Coordinates`.
- `shapes/` – traits `Transform`, `Intersect`, `Shape`; impls `Sphere`, `Plane`, `Cube`, `Cylinder`, `Cone`, `Triangle`.
- `patterns/` – trait `ColorProvider` (what a `Material` holds), traits `Pattern` (3D, point-based) and
  `Texture` (UV-mapped via `mapping::Mapping`); impls stripe/gradient/ring/checker/complex patterns.
  Textures (`checkered_texture`, `custom_texture`, `mapping`) are **work in progress** and not yet exported.
- `composites/` – `Ray`, `Intersection(s)`, `ComputedHit`, `Material`, `World`, `Camera`, `Canvas`.
- `consts.rs` – `EPSILON`, `PI`, etc. `utils.rs` – `CoarseEq`, `Squared`, `Cubed`, `solve_quadratic`, test fixtures.
- `dyn_partial_eq.rs` – sealed `DynPartialEq` enabling `==` on `dyn Shape` / `dyn ColorProvider`.

Render pipeline: `Camera::render[_parallel]` → `Camera::ray_for_pixel` → `World::color_at`
→ `World::collect_intersections` (each `Shape::local_intersect` in object space) → `ComputedHit`
→ `Material::lighting` (+ recursive reflection/refraction) → `Canvas`.

## Commands

Run from the repo root.

```sh
cargo build --workspace
cargo test --workspace                     # unit + doc tests, ~seconds
cargo test --workspace --release           # CI also runs this; float/overflow behaviour can differ
cargo test -p ray-tracer <filter>          # one crate / name filter
cargo clippy --workspace --all-targets
cargo fmt --all --check
cargo doc --workspace --no-deps

# End-to-end render (ALWAYS --release; debug builds are extremely slow)
cargo run --release -p ray-tracer-cli -- scenes/three_sphere_scene.yaml target/renders/out.png

# Benchmarks (~45 s; see Performance)
cmd //c "scripts\bench.cmd"
```

Scenes in `scenes/` are mostly 4K and can take a long time. For a quick end-to-end check, copy a scene
into `target/renders/`, lower the camera `width`/`height` (e.g. 400×300), and render that copy.

## Definition of done

Before declaring a change complete:
1. `cargo build --workspace` has **no new warnings** in the code you touched.
2. `cargo test --workspace` passes.
3. `cargo clippy --workspace --all-targets` reports nothing new in the code you touched.
4. `cargo fmt --all` has been run on files you changed (do not reformat unrelated files).
5. For changes that affect rendered output, render a small scene copy (see Commands) and look at the image.
6. For changes in the hot path (see Performance), compare the relevant benchmarks against a baseline
   recorded before the change.

## Code conventions

These are deliberate project style. Follow them even where clippy defaults or "idiomatic Rust" differ.

- **Explicit `return` everywhere** — every function body ends in `return …;`, not a tail expression
  (closures excepted). `clippy::needless_return` is allowed at workspace level for this reason.
- **Numeric constructors take `impl Into<f64>`**: write `Point::new(0, 1, 0)`, `Color::new(0.8, 1, 0.6)`,
  `transformations::scaling(2, 2, 2)` — no `0.0`/`as f64` noise needed.
- **Arithmetic on `Point`/`Vector`/`Color` is by value**: `a + b`, `point - vector`, `color * 0.5`,
  `transformation * point`. These are small `Copy` types (24 bytes; `Matrix<4>` is 128), so write
  `*point - *vector` when you hold references. Do **not** add `impl Add for &Point`-style operators: a
  reference-based version was benchmarked 4–7% slower with identical output, and it forces noisy
  `&(&a + &b) * 2.0` call sites. See [Performance](#performance).
- **Group scalars before scaling a tuple**: `vector * (2.0 * d)` rather than `vector * 2.0 * d`
  (one tuple multiply instead of two).
- **Float comparison**: use `CoarseEq::coarse_eq` / `EPSILON` from `consts`, never raw `==` on computed `f64`.
  `PartialEq` on `Point`/`Vector`/`Color`/`Matrix` is derived, i.e. **exact**; `assert_eq!` on computed
  values only works when the arithmetic is exact, otherwise assert `a.coarse_eq(&b)`.
- **Shapes and patterns store `transformation_inverse`**, not the transformation. Implement `Transform`
  with the `impl_transform_for!(Type)` macro rather than by hand.
- **Trait objects**: shapes live in `World` as `Box<dyn Shape>`; patterns in `Material` as
  `Option<Arc<dyn ColorProvider>>`. New implementors must derive `PartialEq` (blanket `DynPartialEq`),
  and implement `Debug`, `Display`, `Send + Sync`.
- **No `unsafe`** (advertised in README). Avoid `unwrap()` in non-test library code; the CLI propagates
  errors with `Result<_, Box<dyn Error>>`.
- Doc comments with runnable examples (`/// # Examples`) for public constructors are encouraged; they run
  as doctests.
- Avoid heap allocations and cloning of large or heap-owning data (`Material`, `World`, `Vec`s) in the hot
  render path (`World::color_at` and below) — borrow those. Small `Copy` primitives are fine to copy.

## Performance

The hot path is `Shape::local_intersect`, `Shape::normal_at`, `Material::lighting` and the `World`
recursion. Any change there is a performance change and must be measured — intuition is often wrong:

- **Measured: by-value operators beat by-reference** for `Point`/`Vector`/`Color` (see Code conventions).
- **Measured: seed `f64` min/max chains with a non-NaN constant.** `MIN.max(a).max(b).max(c)` (or
  `fold(MIN, f64::max)`) is ~8% faster on cube-heavy scenes than `a.max(b).max(c)` — likely because the
  known non-NaN seed lets LLVM drop NaN handling. Applies to every multi-way `min`/`max` (`Cube`, AABBs).

### Benchmarks (Criterion, `ray-tracer/benches/`)

`primitives.rs` (vector/color/matrix ops) and `shapes.rs` (`intersect/<shape>/{hit,miss}`, `normal_at`,
`intersections/hit`); fixed-seed inputs in batches of 1024. Full run ~45 s. Always use the script — it
pins the run to one core; unpinned, identical code varied by up to 50%, pinned by ≤6%:

```sh
cmd //c "scripts\bench.cmd -- --save-baseline before"   # before the change
cmd //c "scripts\bench.cmd -- --baseline before"        # after; add a filter, e.g. intersect/cube
```

"Regressed"/"improved" means beyond the 5% noise threshold; re-run before trusting smaller changes.
Close IDE indexing/games first, and only compare baselines from the same machine.

### Whole-render comparisons

1. Build both variants with `cargo +stable build --release -p ray-tracer-cli` into separate
   `CARGO_TARGET_DIR=target/bench-<name>` dirs (never `%TEMP%`; without `rust-toolchain.toml` the `esp`
   default toolchain is used and results are meaningless).
2. Alternate `-r serial` renders of a ~960×640 scene copy, ≥7 each; compare medians (<3% is noise).
3. Check the PNGs are byte-identical (`cmp`) unless the image is meant to change.

## Tests

Put a test where its subject lives:

| Kind | Where | Use for |
|---|---|---|
| Unit | `#[cfg(test)] mod tests { use super::*; … }` at the end of the file | One type or function: primitives, a single shape's intersections/normals, a pattern, private helpers (`World::shade_hit`, `Matrix::submatrix`, …). May use private items. |
| Integration | `ray-tracer/tests/integration/` (one binary: `main.rs` + a module per topic) | Public-API behaviour spanning modules: rendering a `World` through a `Camera`, whole-pipeline/golden-image checks, trait-object behaviour, file output. |
| CLI end-to-end | `ray-tracer-cli/tests/` | Running the binary (`env!("CARGO_BIN_EXE_ray-tracer-cli")`) on small scene files: exit codes, error messages, output images. |
| Doctest | `/// # Examples` on public items | Showing how to call a public constructor or function. |

- If a test needs private items it is a unit test — never make something `pub` only to test it from `tests/`.
- Keep integration tests in the single `integration` binary rather than adding files directly under
  `tests/`: every top-level file is a separate crate to compile and link (with LTO under `--release`).
- The CLI is a binary-only crate, so the scene loader's own tests stay inline in `scene_loader.rs`.
- Files written by tests go under `env!("CARGO_TARGET_TMPDIR")` (inside `target/`), never the repo or `%TEMP%`.
- Many tests mirror the book's Cucumber scenarios; name them after the scenario (`ray_misses_cube`, …).
- Use `rstest` (`#[rstest] #[case(…)]`) for table-driven tests — see `shapes/cube.rs`, `shapes/cone.rs`.
- Shared fixtures (e.g. default-world spheres) live in `utils.rs`; `World::default()` is the book's default
  world. Integration-only helpers go in `tests/integration/support.rs`.
- Every bug fix should come with a test that fails without the fix.
- **Golden images** (`tests/integration/golden.rs`) compare small renders with the PNGs in
  `tests/integration/golden/` (±2 per channel, up to 0.1% outlier pixels). A failure saves the new
  render under `target/tmp/integration/`. If the change in output is intended, regenerate with
  `UPDATE_GOLDEN=1 cargo test -p ray-tracer --test integration golden`, look at the images, and commit them.

## Recipes

**Add a shape** (model on `shapes/plane.rs`):
1. `shapes/<name>.rs`: struct with `pub material: Material` and private `transformation_inverse: Transformation`;
   `impl_transform_for!`; `impl Intersect` (object-space ray, push into `intersections`);
   `impl Shape` (`local_normal_at`, `material`); `Default`, `Display`, `CoarseEq`.
2. Register `mod`/`pub use` in `shapes.rs`.
3. Optionally make it loadable from YAML: add a `ShapeKind` variant and its `add:` name in
   `scene_loader/format.rs` (`Entry` deserializer), build it in `scene_loader/resolve.rs` (`Definitions::shape`).

**Add a pattern** (model on `patterns/stripe_pattern.rs`): struct + `impl_transform_for!` +
`impl_color_provider_for_pattern!` + `impl Pattern` + `Display`; export in `patterns.rs`;
for YAML, add a `PatternKind` variant (`format.rs`) and handle it in `Definitions::pattern` (`resolve.rs`).

**Scene files** (`ray-tracer-cli/src/scene_loader/`): `format.rs` is the serde model of the YAML (one type
per mapping, `deny_unknown_fields`, so new keys must be added there), `resolve.rs` turns it into a `World`.
Entries start with `add:` (camera, light, shapes) or `define:` (a material, transform list or `[r, g, b]`
color, recognised by the value's shape); materials may `extend` another. Transform steps apply in list
order. Errors carry line/column; `every_example_scene_loads` keeps `scenes/*.yaml` valid.

## Gotchas

- Windows dev machine with `core.autocrlf=true`; files are LF in the repo. Don't mass-convert line endings.
- CI is `.github/workflows/ci.yml` (hand-written YAML): `cargo fmt --check`, `cargo clippy` and debug +
  release tests on Linux, Windows and macOS. Keep it in sync when the commands above change. The
  toolchain comes from `rust-toolchain.toml` (via `actions-rust-lang/setup-rust-toolchain`); warnings
  don't fail CI yet (`build-warnings: warn`) — remove that once the existing warnings are fixed.
- `.cargo/config.toml`'s `target-cpu=native` is intentionally disabled (`cfg(any())`).
- Keep all build output, benchmark builds, scratch renders and
  temporary worktrees under the repo's `target/` (excluded from scanning); never build or run binaries
  from `%TEMP%` or other new directories. Reuse target dirs instead of creating fresh ones, and prefer
  `cargo check` when you only need type-checking. `.cargo/config.toml` points `TMP`/`TEMP` at `target/`
  for everything cargo runs, so rustdoc's doctest binaries (`rust_out.exe`) and linker temp files stay
  there too; don't remove that `[env]` block. Leave a temporary worktree (`cd` out) before deleting it,
  or Windows keeps the folder locked.
- The machine's rustup default toolchain is `esp`; only `rust-toolchain.toml` pins `stable`. Use
  `cargo +stable …` anywhere that file is absent (temp dirs, old worktrees).
- The default branch is `master`.

## Git

- Commit messages: short imperative sentence-case summary, e.g. `Add Texture trait, rework Pattern trait`.
- Don't commit renders, `target/`, or IDE files. Don't overwrite `rendered_images/` unless asked.
- Only commit/push when asked; never rewrite published history.
