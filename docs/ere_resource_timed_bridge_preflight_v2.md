# Corrected timing-bridge preflight v2

The first frozen package `output/ere_resource_timed_bridge_20261008` is failed,
sealed and zero-native:695 entries, root
`560ea51646703e3b3c10bd68e57b0e3637701796a1787767362b315316716e82`.
Rust preflight passes; Python preflight rejects its own refusal-slot IDs
32/34/36/38. Its lazy generator read a growing list length during `extend`.
It never launched physics or fitting.

Change that generator to an eagerly evaluated list so the four intended IDs
are32/33/34/35, matching the already frozen Rust schedule and36-slot contract.
The complete synthetic accounting and corruption checks now pass, with the
real-native group explicitly skipped until physics exists. Do not change
native code, policy, parameters, draws, criteria, denominator, sources of
utility or learning proposal. Preserve the failed package and record the exact
one-line verifier diff plus revised wrapper provenance before/after.

Freeze the separately registered
`output/ere_resource_timed_bridge_preflight_v2_20261008`, requiring the same
compiled Rust binary SHA256
`f863f9d739aa86abccf998973a2178eaa2acf30030d4ed00f84e962c95d11eb3`.
Only after corrected frozen preflight passes may this package execute the first
32-cell/four-refusal native engineering panel. Its maximum1,063 advances/35
Seal and zero updates are unchanged. This is no native retry or outcome-selected
campaign; the old failed manifest is not promoted.
