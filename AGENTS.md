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
| `github-workflow/` | **not** a workspace member | `build.rs` that generates `.github/workflows/ci.yml` via the `gh-workflow` crate. |

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
cargo fmt --all --check                    # CI uses `cargo +nightly fmt --check`; output is currently identical
cargo doc --workspace --no-deps

# End-to-end render (ALWAYS --release; debug builds are extremely slow)
cargo run --release -p ray-tracer-cli -- scenes/three_sphere_scene.yaml target/renders/out.png
```

Scenes in `scenes/` are mostly 4K and can take a long time. For a quick end-to-end check, copy a scene
into `target/renders/`, lower the camera `width`/`height` (e.g. 400×300), and render that copy.

## Definition of done

Before declaring a change complete:
1. `cargo build --workspace` has **no new warnings** in the code you touched.
2. `cargo test --workspace` passes.
3. `cargo clippy --workspace --all-targets` reports nothing new in the code you touched.
4. `cargo fmt --all` has been run on files you changed (do not reformat unrelated files).
5. For changes that affect rendered output, render a small scene and look at the image.

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
  `PartialEq` on primitives is already epsilon-based.
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

How to benchmark (renders print their own timing):
1. Build both variants with the **same toolchain** (`cargo +stable build --release -p ray-tracer-cli`) and
   separate target dirs **inside `target/`** (`CARGO_TARGET_DIR=target/bench-<name>`, worktrees in
   `target/worktrees/`) — never in `%TEMP%` (see Gotchas: antivirus). A worktree without the committed `rust-toolchain.toml` silently uses the
   machine's rustup default (here the `esp` fork), which once produced a fake 30% "speedup".
2. Use a scene copy at ~960×640 and `-r serial` (less noise than parallel).
3. Alternate the variants, ≥7 runs each, compare medians. Differences under ~3% are noise/code layout.
4. Confirm output PNGs are byte-identical (`cmp`) unless the change is meant to alter the image.

## Tests

- Unit tests live inline in each file under `#[cfg(test)] mod tests { use super::*; … }`.
- Many tests mirror the book's Cucumber scenarios; name them after the scenario (`ray_misses_cube`, …).
- Use `rstest` (`#[rstest] #[case(…)]`) for table-driven tests — see `shapes/cube.rs`, `shapes/cone.rs`.
- Shared fixtures (e.g. default-world spheres) live in `utils.rs`; `World::default()` is the book's default world.
- Every bug fix should come with a test that fails without the fix.

## Recipes

**Add a shape** (model on `shapes/plane.rs`):
1. `shapes/<name>.rs`: struct with `pub material: Material` and private `transformation_inverse: Transformation`;
   `impl_transform_for!`; `impl Intersect` (object-space ray, push into `intersections`);
   `impl Shape` (`local_normal_at`, `material`); `Default`, `Display`, `CoarseEq`.
2. Register `mod`/`pub use` in `shapes.rs`.
3. Optionally wire it into `ray-tracer-cli/src/scene_loader.rs` (`parse_scene` match) and add a scene.

**Add a pattern** (model on `patterns/stripe_pattern.rs`): struct + `impl_transform_for!` +
`impl_color_provider_for_pattern!` + `impl Pattern` + `Display`; export in `patterns.rs`;
add to `SceneParser::parse_pattern` if it should be usable from YAML.

**Scene YAML keys** are handled in `ray-tracer-cli/src/scene_loader.rs` (`add:`, `define:`, `extend:`,
`material:`, `transform:` with `scale`/`translate`/`rotate-x|y|z`). Definition names are typed by suffix:
`*-color`, `*-material`, `*-transform` / `*-object`.

## Gotchas

- Windows dev machine with `core.autocrlf=true`; files are LF in the repo. Don't mass-convert line endings.
- `.github/workflows/ci.yml` is **generated** — edit `github-workflow/build.rs` and regenerate instead.
  `.github/workflows/tests.yaml` is the hand-written workflow currently used by the README badge.
- `.cargo/config.toml`'s `target-cpu=native` is intentionally disabled (`cfg(any())`).
- Keep all build output, benchmark builds, scratch renders and
  temporary worktrees under the repo's `target/` (excluded from scanning); never build or run binaries
  from `%TEMP%` or other new directories. Reuse target dirs instead of creating fresh ones, and prefer
  `cargo check` when you only need type-checking.
- The machine's rustup default toolchain is `esp`; only `rust-toolchain.toml` pins `stable`. Use
  `cargo +stable …` anywhere that file is absent (temp dirs, old worktrees).
- The default branch is `master`.

## Git

- Commit messages: short imperative sentence-case summary, e.g. `Add Texture trait, rework Pattern trait`.
- Don't commit renders, `target/`, or IDE files. Don't overwrite `rendered_images/` unless asked.
- Only commit/push when asked; never rewrite published history.
