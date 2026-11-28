# Repository guidance

## Start with the README

Read `README.md` first. It is the starting point for understanding the
architecture, local development workflow, deployment model, and configuration.
Keep `README.md` up to date in the same change whenever behavior, architecture,
configuration, commands, or deployment requirements change.

## Code organization and language

- Prefer several small, focused files over a few monolithic files. Split code by
  component, domain, or responsibility rather than by arbitrary line counts.
- Rust is the default implementation language. Use JavaScript only when the
  browser or build environment requires it or when implementing the behavior in
  Rust is impractical.
- Reuse existing modules and patterns before adding new abstractions.
- Keep user-facing frontend behavior in the Rust frontend; reserve direct
  JavaScript interop for APIs that are not reasonably available through Rust or
  the existing web crates.

## Versions and changelog

Consider a version bump and a `CHANGELOG.md` entry for every feature
implementation and bug fix. Do not defer this review to release time: resolve it
as part of the implementation. A purely internal refactor, documentation-only
change, or test-only change may not need a bump, but it should still be
considered explicitly.

The application release version is stored in both application crates:

- `frontend/Cargo.toml`: drives `APP_VERSION` and the PWA-visible version.
- `backend/Cargo.toml`: identifies the matching server release.

Keep those versions synchronized for a new application release. Choose the
version according to Semantic Versioning: patch for backwards-compatible fixes,
minor for backwards-compatible features, and major for breaking changes. Before
1.0, apply the project's `0.x` convention consistently and treat breaking
changes as a minor-version change. Do not bump the shared or migration crate
versions merely to follow an application release; only change them when their
own package/release semantics require it.

When a bump is needed:

1. Update `frontend/Cargo.toml` and `backend/Cargo.toml` to the new version.
2. Run a Cargo command such as `cargo check --workspace` so `Cargo.lock` records
   the workspace package versions.
3. Move or add the release notes in `CHANGELOG.md` under that version and the
   release date. Keep an `Unreleased` section at the top for future work.
4. Verify that the version exposed by the frontend and the server package is
   consistent.

Use clear, user-focused changelog entries for every noteworthy feature or fix.
Organize entries under `Added`, `Changed`, `Fixed`, `Removed`, or `Security` as
appropriate. Record compatibility, migration, and configuration impacts, not
implementation trivia. Until a release is cut, place new entries in
`Unreleased`; when cutting a release, create a dated heading for the new version
and retain a fresh `Unreleased` section above it.

## Completing a change

- Run focused tests and `cargo fmt` for Rust changes.
- Run the smallest relevant Cargo checks or tests, using
  `cargo check --workspace` when a change crosses crate boundaries.
- Ensure `README.md` and `CHANGELOG.md` still describe the resulting behavior
  accurately.
- Review versioning as part of every feature or bug fix before considering the
  work complete.
