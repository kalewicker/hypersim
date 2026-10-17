# Resource adapter corrected accounting v2

The first engineering package `output/ere_resource_adapter_20261008` failed in
its Python verifier before auditing inputs: its regular expression did not
escape the literal parentheses in NodeId/EdgeId strings. It remains sealed
failed (681 entries), root
`a34244da71bc81d656e1979b55ac0fa75ee0ec6cd4689ab0d1135028aec38c38`.
The three Rust groups and saved-receipt fixture passed; no native/optimizer
work occurred. This is not an admissible completed v1 audit.

Separately freeze corrected accounting under
`output/ere_resource_adapter_accounting_v2_20261008`. Change only the verifier's
NodeId/EdgeId patterns and matching capture indices; add direct malformed-key
regressions to its existing Python group. Keep Rust adapter/tests, config,
inputs, neutral/probe parameters, utility, denominators and tolerances byte
identical to v1. The wrapper records/preserves the failed root before/after and
records a verifier diff. No outcome-dependent training retry is involved:
this separate engineering correction runs zero native/optimizer work and
cannot promote the original failed manifest or scientific flags.
