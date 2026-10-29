# Instructions for AI Agents and Contributors

These instructions apply to the entire repository. A more specific `AGENTS.md` in a subdirectory may add constraints for that subtree. Keep user changes intact, inspect the current Git state before editing, and never claim work or validation that was not performed.

## Project purpose and source of truth

Kvas is a Rust game-engine workspace with an editor, player, renderer, ECS, audio, sample games, and a bilingual VitePress site. Read `README.md`, `DEVELOPMENT.md`, `CONTRIBUTING.md`, and the relevant crate documentation before making broad changes. `ROADMAP.md` distinguishes implemented, partial, and planned features; do not describe a planned feature as shipped.

The code is authoritative for actual behavior. Verify API names, file paths, feature flags, platform support, performance claims, and generated output against the checkout before documenting them. If a task changes behavior, update both English and Russian user-facing documentation when a corresponding page exists. Keep examples executable or covered by tests where practical.

## Before editing

1. Read `git status --short --branch` and inspect the relevant files and tests. Do not overwrite or discard existing user work, including untracked files.
2. Find the owning crate and existing patterns before adding a dependency, abstraction, API, or parallel implementation.
3. Check `Cargo.toml`, crate-level instructions, feature gates, supported targets, generated-file headers, licensing, and the relevant docs.
4. For a bug, reproduce or add a focused regression test before changing behavior where feasible. For a feature, define observable acceptance criteria and cover success, invalid input, and boundary behavior.

## Implementation standards

- Keep changes scoped, idiomatic, and compatible with the existing architecture. Prefer small composable changes; avoid unrelated refactors.
- Preserve the engine's shared paths: `Field` reflection for editor/serialization/undo integration, `Assets` for project and packaged asset access, the shared `Runtime` for editor play and player execution, and the shared renderer.
- Treat public Rust API changes as compatibility changes: document arguments, units, lifecycle, errors, and platform/feature requirements. Add examples and migration notes when callers may be affected.
- Handle errors at subsystem boundaries. Do not add unchecked panics, silent fallbacks, data loss, or hidden behavior to make a test pass.
- Keep rendering and GPU work compatible with supported wgpu backends. Gate optional vendor SDKs and hardware-dependent functionality explicitly; do not imply SDK branding or hardware support without a real integration and evidence.
- Keep UI work consistent with existing localization, themes, keyboard navigation, accessibility, saved state, and small-window behavior. Add translation keys rather than hard-coded user-facing strings.
- Avoid adding dependencies unless existing facilities cannot reasonably solve the problem. Explain material size, license, maintenance, and platform costs in the change.
- Respect asset provenance. Never add imported assets unless redistribution rights and required attribution are clear. Keep secrets, credentials, machine-local data, build output, recordings, and private user content out of Git.
- Use repository-relative paths for project dependencies and resources. Never commit absolute paths tied to a developer machine; validate projects from a clean clone or CI checkout.

## Tests, coverage, and performance

Every behavior change must include tests at the narrowest useful layer and all affected layers. A change is not complete merely because it compiles. Before **every commit**, run:

```powershell
./tools/pre-commit.ps1
```

Install `cargo-llvm-cov` once with `cargo install cargo-llvm-cov --locked`. The repository's `.githooks/pre-commit` invokes this gate after `git config core.hooksPath .githooks` has been configured. The gate runs all workspace targets under LLVM line coverage as well as compilation and doctests. GitHub Actions repeats coverage and validation for each pull request and retains the LCOV report. Do not skip the hook or required CI checks to land a change.

Add tests for regressions, boundary conditions, serialization compatibility, failure paths, and user-visible behavior as applicable. Shader and node changes must validate shader compilation across relevant modes. Script examples and generated catalogues must be regenerated and checked with their owning generator and tests. GPU-specific behavior needs an explicit adapter-backed test or a clearly documented manual validation; CPU-only CI does not prove GPU correctness.

For any performance optimization:

1. State the target workload, hardware/backend, build profile, and baseline measurement.
2. Add or extend a deterministic benchmark or repeatable workload before optimizing. Record warm-up, sample count, and variance; avoid asserting fragile wall-clock timings in ordinary unit tests.
3. Add correctness/regression tests proving output and behavior remain valid, plus a benchmark comparison demonstrating the intended improvement and checking for regressions in related workloads.
4. Report before/after numbers and measurement limits in the pull request. If no trustworthy baseline or benchmark harness exists, add the harness or mark the result unverified; do not claim a speedup from intuition.

Coverage should grow with changed behavior. CI publishes a coverage report as evidence; until a stable repository-wide baseline and target-specific exclusions are agreed, do not invent a global percentage or weaken tests to reach one. Add tests for changed lines and newly introduced branches, and call out any code that cannot be covered (for example, hardware-specific paths).

## Documentation is part of the change

Update documentation in the same change as code. At minimum, consider:

- the relevant crate/module rustdoc and public API examples;
- English and Russian user guides, editor workflows, scripting/material/audio pages, and generated references;
- README/setup instructions when installation, requirements, features, or commands change;
- `ROADMAP.md` when feature status, limitations, dependencies, or priorities change;
- changelog/release notes for user-visible changes and breaking changes.

Document installation, prerequisites, exact launch/build steps, expected output, configuration, limitations, and recovery steps for new workflows. Keep generated docs generated by their canonical tools; edit the source and rerun the generator instead of hand-editing generated output. Do not mark documentation complete based only on adding a short API comment.

## Git and pull request workflow

- Use focused commits with imperative subjects, e.g. `Add asset import validation` or `Docs: clarify Linux setup`.
- Do not commit unless the user requested a commit or it is necessary to fulfill the explicitly requested workflow. Never amend, force-push, or rewrite shared history unless explicitly asked.
- Inspect `git diff --check`, `git diff`, and `git status` before committing. Stage only intended files and inspect the staged diff for secrets, build output, generated churn, large assets, and licensing.
- Pull requests must explain the problem, implementation, tests and coverage, performance evidence when relevant, docs updated, limitations, and screenshots/captures for visual changes.
- Keep PRs reviewable. Separate unrelated changes and generated bulk output unless the task specifically requires them.
- Report commands and results exactly. Distinguish local validation, CI, push, and published deployment; a successful push is not proof that CI or Pages deployment succeeded.

## Completion checklist

- [ ] The requested behavior or documentation is implemented and consistent with actual code.
- [ ] Tests cover changed behavior, regressions, errors, and important boundaries.
- [ ] Performance work includes a repeatable baseline, correctness tests, and before/after evidence.
- [ ] User and developer documentation is updated in both languages where applicable.
- [ ] Generators, formatting, and pre-commit verification pass, or remaining failures are identified precisely.
- [ ] The final diff contains only intended, licensed, non-sensitive project files.
- [ ] The handoff states what changed, what was verified, and anything still blocked or unverified.
