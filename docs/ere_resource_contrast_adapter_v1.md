# Fixed graph-cue contrast adapter: zero-native preflight v1

Preserve the negative learning campaign and its diagnostic. Add a default-off
train-layer module only; native graph projection, timing, physics, payments,
old adapter/runner bodies and frozen proposal remain unchanged. The one existing
source change is an additive module export. No training or native work is allowed.

Use the existing strict step-13 graph-token observation/domain checks. Four actor
parameters start at zero. From validated forecast {0,1} and bounty {2,12}, define
`f=2*forecast-1`, `b=2*(bounty-2)/10-1`. Full input is `[1,f,b,f*b]`. ForecastMasked
is `[1,0,b,0]`; BountyMasked is `[1,f,0,0]`; CuesMasked is `[1,0,0,0]`.
Apply masks after transformation; missing cues are not coded as false low values.
Reject invalid domains even when masked. Validate signed head patterns, finite
parameters bounded by eight, action/draw/utility domains and cross-term identity.

Wallet is no longer an actor feature. It remains in the validated observation,
admission and zero-unpaid-credit mask. Pure `choose` returns a step-13 request
for the existing sim-core bridge, or None when unfunded; it executes no physics.
Sample every unpaid case and retain its outcomes. Actor contribution uses the
same funded `(actual_utility-9.5)/20 * score_gradient` or zero when unpaid.
Never infer funding from a contrast coordinate. The old runner's input[1]-based
credit mask and nonnegative-input head are incompatible with this new basis:
a future runner must be separately versioned, not silently patched or reused.

Freeze two arithmetic probes (neutral and existing [0.07,-0.11,0.13,-0.17]), all
eight states/four masks, both actions and two draws (0.25/0.75). Authenticated
saved receipts supply actual native utilities only for diagnostic arithmetic.
Check 64 probe records, 128 scores and 128 pure requests, independent derivative
and utility/contribution accounting. No score gradient or expected batch vector
may be applied to train parameters.

Check the four funded Full inputs have Gram matrix 4I/rank four; masked rank is
one. Enumerate the expected eight-state batch gradient independently, retaining
zero unpaid actor credit and denominator eight. At each fixed probe, Full state's
raw expected logit direction must equal `p*(1-p)*(Ud-Uh)/40`, with the correct
sign in all four funded states. This is an expectation/coordinate property, not
a guarantee for stochastic finite batches or a training/scientific gate pass.
Report partial-mask aliasing, feature-norm changes and actor/funding separation.

Test provenance invariance, domain/malformed graph/parameter/draw/action refusals,
the funded-safe and funded-masked cases, and synthetic unpaid utility perturbation
so zero credit cannot be explained only by native equal-action utilities. Run
the old adapter/timing/learning normal tests and both feature-off checks.

Use live zero-native scratch checks before final freezing. Seal a fresh package
below 128 MiB with full source closure, exact test executable, copied inputs and
source/build/test/audit/preservation receipts. Independently recheck from a
relocated copy without live-source/input fallback. No new optimizer, curriculum,
seed selection, checkpoint selection, native campaign or admission change is
authorized by this engineering milestone. Next, if this passes, is a separately
versioned runner and frozen matched protocol before launch approval.
