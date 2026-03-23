This fixture vendors the ESM source set from `three@0.183.2`.

Contents:
- `src/**/*.js`
- top-level ESM modules and directories such as `Addons.js`, `controls/`, `loaders/`, `shaders/`
- package `LICENSE`

The files are checked in directly under this directory as unpacked sources:
- `src/`
- the flattened former `examples/jsm/` tree
- `LICENSE`

This keeps benchmark fixture changes reviewable in git without going through a tarball.
