# Final engineering verifier: explicit import correction

Registered 2026-10-02 before this continuation. Preserve the v1, v2 and v3
engineering failure archives and their frozen sources. V3 stopped before
regressions/controls because its copied verification function lacked three
standard-library imports (`math`, `sys`, `json`); the recorded exception was
`NameError: math`. This is a verifier implementation defect, not a scientific
or resource result. Add all three imports in a new source version and add two
symbol-table regression groups to detect unresolved global names and reproduce
that missing-import pattern. Do not edit the frozen v3 script or failed receipt.

All resource workers are reused from the unchanged v1 archive, without repetition
or selection. Accounting equivalence, source-stage fixtures, 14 detached controls,
RSS thresholds, wall limits and native/scientific boundaries stay exactly those
of the v3 contract. Run 27 regression groups: two import guards plus the previous
25. Freeze all twelve combined engineering sources before checks; verify all
three failure roots, both native input roots, registered inputs and all 472 live
historical source hashes afterward. Preserve any failure at its fresh destination.

No new native work, resource-worker rerun, seeds, grammar compilation, training,
scientific launch, criteria change, native-source change or launch-gate waiver.
This receipt certifies the supplemental storage optimization only; integrating
it into a prospectively versioned scientific launcher remains separate work.
