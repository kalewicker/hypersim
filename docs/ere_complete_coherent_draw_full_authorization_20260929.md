# Complete coherent full assay: launch authorization

Date: 2026-09-29. Profile `E2-complete-coherent-draw-assay-v1`.

The user explicitly replied **“I approve it”** to the proposal for one
10,000-draw × 100-step scientific trial under the fixed preregistered caps.
This authorizes exactly one fresh invocation at
`output/ere_complete_coherent_draw_full_20260929`.

Approved envelope: at most 1,016,800 attempted/advancing Engine steps;
one producer thread; 3,600 total producer wall seconds including setup/freeze,
controls, trajectories, audits and sealing; 900 shared auditor seconds with
a 120-second freeze sublimit; 4 GiB total output; 1 GiB auditor RSS;
Pooled classification matching at 400,000 work / 50,000 results per step;
2,000-node / 4,000-edge store caps. Runtime limits remain unchanged.

Contract Markdown SHA256:
`8a0e97586c4c7f84c28c105b246ffaeb8688618ae659c9ea3716ed5b7bba1d46`.
Contract JSON SHA256:
`80c8cbc072a393d8bddd921b5c4dbf88ac60e8e4b1ebb50dfb7a0683aa967dba`.

The full-size freezing cost and tight wall headroom were disclosed before
approval. No budget increase, scientific criterion change, automatic retry,
alternative seed, historical rewrite or training is authorized.
Preserve a capped/failed/incomplete archive and report its actual status.
Preparation documents remain historical records of the pending approval state;
this separate authorization supersedes that launch boundary only.

The launcher passes the exact approved step/cap/JSON-contract tokens.
The producer creates its own frozen `launch_authorization.json` before
measurement and the frozen auditor verifies that attestation.
