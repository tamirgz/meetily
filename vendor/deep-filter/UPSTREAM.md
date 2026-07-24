# Vendored DeepFilterNet runtime

This directory vendors the official `libDF` Rust inference crate and the
`DeepFilterNet3_onnx.tar.gz` model from:

- Repository: <https://github.com/Rikorose/DeepFilterNet>
- Upstream revision: `d375b2d8309e0935d165700c91da9de862a99c31`
- Upstream crate version: `0.5.7-pre`

The upstream MIT and Apache-2.0 license files are preserved in `libDF/`.

## Local compatibility patch

Meetily uses Tract `0.21.10`. The upstream crate declares `^0.21.4`, but:

- Tract `0.21.4` rejects the bundled DFN3 decoder during graph compaction with
  a duplicate-node-name error.
- Tract `0.21.10` moved its public ndarray types to ndarray `0.16` and renamed
  `InferenceModel.symbol_table` to `InferenceModel.symbols`.

The vendored copy therefore makes only these compatibility changes:

1. pin the four Tract crates to `0.21.10`;
2. update ndarray from `0.15` to `0.16`;
3. replace the three `m.symbol_table.sym("S")` calls with
   `m.symbols.sym("S")`.

The runtime test
`deepfilternet_initializes_and_returns_finite_audio` must pass before updating
either the upstream revision or Tract version.
