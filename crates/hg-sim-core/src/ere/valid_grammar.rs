//! Finite complete-family productions typed before stochastic emission.
//! No physics, fitness-conditioned selection, rejection sampling or parent fallback.
use super::{
    coherent::{self, Request, Role, Status},
    meta_excluded_store, semantic,
    signal::{self, Emission},
};
use hg_core::{HypergraphStore, NodeId};
use hg_meta::{encode_complete_family, read_complete_program, MetaLimits, RuleProgram};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use serde_json::{json, Value};
use std::collections::BTreeMap;

pub const PROFILE: &str = "E2-complete-valid-production-grammar-v1";
pub const PARAMS: usize = 183;
pub const OPERATORS: [&str; 13] = [
    "AddSlot",
    "RemoveSlot",
    "RetargetSlot",
    "AddVar",
    "RemoveVar",
    "AddPatternEdge",
    "RemovePatternEdge",
    "AddLabel",
    "RemoveLabel",
    "ChangeLabel",
    "ChangePriority",
    "Activate",
    "Deactivate",
];
type Res<T> = Result<T, String>;
fn hex(bytes: Vec<u8>) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
fn role(index: usize) -> Role {
    if index == 0 {
        Role::Core
    } else {
        Role::Proposal
    }
}
fn structural(c: &hg_meta::CandidateRule) -> String {
    let mut c = c.clone();
    c.priority = 0;
    hex(c.rcf_bytes())
}

struct Production {
    request: Request,
    program: RuleProgram,
}
struct Parent {
    store: HypergraphStore,
    root: NodeId,
    program: RuleProgram,
    groups: Vec<[Vec<Production>; 2]>,
    stats: Vec<[BTreeMap<String, usize>; 2]>,
}
/// Immutable, finite grammar for the fixed complete-profile parent bank.
pub struct Grammar {
    parents: Vec<Parent>,
}

fn requests(s: &HypergraphStore, root: NodeId, target: Role) -> Res<Vec<Vec<Request>>> {
    let t = coherent::targets(s, root, target, &MetaLimits::default())?;
    let mut groups: Vec<Vec<Request>> = (0..13).map(|_| vec![]).collect();
    for &pattern in &t.patterns {
        for &variable in &t.variables {
            groups[0].push(Request::AddSlot { pattern, variable });
        }
        for slot in coherent::slots(s, pattern) {
            groups[1].push(Request::RemoveSlot { slot });
            for &variable in &t.variables {
                groups[2].push(Request::RetargetSlot { slot, variable });
            }
        }
        groups[3].push(Request::AddVar { pattern });
        groups[6].push(Request::RemovePatternEdge { pattern });
    }
    for &variable in &t.variables {
        groups[4].push(Request::RemoveVar { variable });
        for value in 0..2 {
            groups[7].push(Request::AddLabel { variable, value });
            groups[9].push(Request::ChangeLabel { variable, value });
        }
        groups[8].push(Request::RemoveLabel { variable });
    }
    for side in [t.left, t.right] {
        for &a in &t.variables {
            for &b in &t.variables {
                groups[5].push(Request::AddPatternEdge { side, a, b });
            }
        }
    }
    groups[10].push(Request::ChangePriority);
    if target == Role::Proposal {
        groups[11].push(Request::Activate);
        groups[12].push(Request::Deactivate);
    }
    Ok(groups)
}

/// Enumerates and types the full domain without consuming an RNG or Engine step.
pub fn compile() -> Res<Grammar> {
    let limits = MetaLimits::default();
    let mut parents = vec![];
    let bank = semantic::parent_bank();
    if bank.len() != 168 {
        return Err("parent bank changed; version the grammar and logit layout".into());
    }
    for (index, (_, program)) in bank.into_iter().enumerate() {
        let mut store = signal::world();
        let root = encode_complete_family(
            &mut store,
            &program.core,
            program.proposal.as_ref().unwrap(),
            program.active,
            &limits,
        )
        .map_err(|v| format!("parent {index}: {v:?}"))?
        .root;
        if read_complete_program(&store.snapshot(), root, &limits)
            .map_err(|v| format!("parent {index}: {v:?}"))?
            != program
        {
            return Err("parent roundtrip mismatch".into());
        }
        let world = meta_excluded_store(&store);
        let mut groups: Vec<[Vec<Production>; 2]> = (0..13).map(|_| [vec![], vec![]]).collect();
        let mut stats: Vec<[BTreeMap<String, usize>; 2]> = (0..13)
            .map(|_| [BTreeMap::new(), BTreeMap::new()])
            .collect();
        for r in 0..2 {
            for (op, reqs) in requests(&store, root, role(r))?.into_iter().enumerate() {
                for request in reqs {
                    let mut candidate = store.clone();
                    let a =
                        coherent::apply(&mut candidate, root, role(r), request.clone(), &limits);
                    let changed = a.after_program.as_ref().is_some_and(|p| p != &program);
                    let outcome = if a.status == Status::Applied && !changed {
                        "NoOp".into()
                    } else {
                        format!("{:?}", a.status)
                    };
                    *stats[op][r].entry(outcome).or_default() += 1;
                    if a.status != Status::Applied || !changed {
                        continue;
                    }
                    let post = read_complete_program(&a.staged.snapshot(), root, &limits)
                        .map_err(|v| format!("typed postcondition {v:?}"))?;
                    post.compile(&limits, false)
                        .map_err(|v| format!("typed compile {v:?}"))?;
                    if a.after_program.as_ref() != Some(&post)
                        || coherent::inventory(&world)
                            != coherent::inventory(&meta_excluded_store(&a.staged))
                    {
                        return Err("type witness/noninterference mismatch".into());
                    }
                    groups[op][r].push(Production {
                        request,
                        program: post,
                    });
                }
            }
        }
        if groups.iter().all(|g| g.iter().all(Vec::is_empty)) {
            return Err(format!("empty parent grammar {index}"));
        }
        parents.push(Parent {
            store,
            root,
            program,
            groups,
            stats,
        });
    }
    Ok(Grammar { parents })
}

/// One softmax draw over explicit support; finite f32 extremes are safe in f64.
fn pick(logits: &[f32], support: &[usize], rng: &mut ChaCha8Rng) -> Res<usize> {
    if support.is_empty() {
        return Err("empty legal support".into());
    }
    let max = support
        .iter()
        .map(|&i| f64::from(logits[i]))
        .fold(f64::NEG_INFINITY, f64::max);
    let weights: Vec<_> = support
        .iter()
        .map(|&i| (f64::from(logits[i]) - max).exp())
        .collect();
    let mut draw = rng.gen::<f64>() * weights.iter().sum::<f64>();
    for (&i, w) in support.iter().zip(weights) {
        if w == 0.0 {
            continue;
        }
        draw -= w;
        if draw <= 0.0 {
            return Ok(i);
        }
    }
    Ok(*support.last().unwrap())
}
impl Grammar {
    pub fn parent_count(&self) -> usize {
        self.parents.len()
    }
    pub fn production_count(&self, parent: usize, op: usize, r: usize) -> usize {
        self.parents
            .get(parent)
            .and_then(|p| p.groups.get(op))
            .and_then(|g| g.get(r))
            .map_or(0, Vec::len)
    }
    pub fn operator_support(&self, parent: usize) -> Vec<usize> {
        self.parents
            .get(parent)
            .map(|p| {
                p.groups
                    .iter()
                    .enumerate()
                    .filter(|(_, g)| g.iter().any(|x| !x.is_empty()))
                    .map(|(i, _)| i)
                    .collect()
            })
            .unwrap_or_default()
    }
    pub fn role_support(&self, parent: usize, op: usize) -> Vec<usize> {
        self.parents
            .get(parent)
            .and_then(|p| p.groups.get(op))
            .map(|g| {
                g.iter()
                    .enumerate()
                    .filter(|(_, x)| !x.is_empty())
                    .map(|(i, _)| i)
                    .collect()
            })
            .unwrap_or_default()
    }
    pub fn emit(&self, theta: &[f32], seed: u64) -> Res<Emission> {
        if theta.len() != PARAMS || theta.iter().any(|v| !v.is_finite()) {
            return Err("expected 183 finite logits".into());
        }
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        let parent = pick(
            &theta[..168],
            &(0..self.parents.len()).collect::<Vec<_>>(),
            &mut rng,
        )?;
        let op = pick(&theta[168..181], &self.operator_support(parent), &mut rng)?;
        let r = pick(&theta[181..], &self.role_support(parent, op), &mut rng)?;
        let count = self.production_count(parent, op, r);
        let production = rng.gen_range(0..count);
        self.emit_selected(parent, op, r, production, seed)
    }
    /// Exact catalogue selection for tests/auditors; unavailable entries are errors.
    pub fn emit_selected(
        &self,
        parent: usize,
        op: usize,
        r: usize,
        index: usize,
        seed: u64,
    ) -> Res<Emission> {
        let p = self.parents.get(parent).ok_or("parent outside grammar")?;
        let production = p
            .groups
            .get(op)
            .and_then(|g| g.get(r))
            .and_then(|g| g.get(index))
            .ok_or("production outside legal grammar")?;
        let mut store = p.store.clone();
        let before = coherent::inventory(&meta_excluded_store(&store));
        let limits = MetaLimits::default();
        let a = coherent::apply(
            &mut store,
            p.root,
            role(r),
            production.request.clone(),
            &limits,
        );
        // A regression cannot silently become rejected sampling or parent fitness.
        if a.status != Status::Applied || a.after_program.as_ref() != Some(&production.program) {
            return Err(format!("grammar/kernel type mismatch: {}", a.receipt));
        }
        let post = read_complete_program(&a.staged.snapshot(), p.root, &limits)
            .map_err(|v| format!("emission postcondition {v:?}"))?;
        post.compile(&limits, false)
            .map_err(|v| format!("emission compile {v:?}"))?;
        if post != production.program
            || coherent::inventory(&meta_excluded_store(&a.staged)) != before
        {
            return Err("emission type/noninterference mismatch".into());
        }
        let signature = hex(post.execution_bytes());
        let classes = json!({"active":post.active,"core":structural(&post.core),"proposal":post.proposal.as_ref().map(structural),"proposal_execution":post.proposal.as_ref().map(|q|hex(q.execution_bytes())),"rcf":hex(post.rcf_bytes())});
        let record = json!({"grammar_profile":PROFILE,"emission_seed":seed,"parent":parent,"operator":op,"role":role(r),"status":"Applied","admitted":true,"signature":signature,"classes":classes,"initial_world_hash":format!("{:016x}",meta_excluded_store(&a.staged).canonical_hash()),"root":p.root,"production_index":index,"production_count":self.production_count(parent,op,r),"operator_support":self.operator_support(parent),"role_support":self.role_support(parent,op),"kernel_attempts":1,"resampled":false,"parent_fallback":false,"kernel":a.receipt});
        Ok(Emission {
            store: a.staged,
            root: p.root,
            program: Some(post),
            record,
        })
    }
    pub fn catalogue(&self) -> Value {
        json!({"profile":PROFILE,"parents":self.parents.iter().enumerate().map(|(i,p)|json!({"parent":i,"root":p.root,"initial_execution":hex(p.program.execution_bytes()),"operator_support":self.operator_support(i),"groups":p.groups.iter().enumerate().map(|(op,g)|json!({"operator":op,"name":OPERATORS[op],"role_support":self.role_support(i,op),"roles":g.iter().enumerate().map(|(r,entries)|json!({"role":r,"stats":p.stats[op][r],"productions":entries.iter().enumerate().map(|(j,e)|json!({"index":j,"request":e.request,"execution":hex(e.program.execution_bytes()),"rcf":hex(e.program.rcf_bytes())})).collect::<Vec<_>>()})).collect::<Vec<_>>()})).collect::<Vec<_>>()})).collect::<Vec<_>>(),"params":PARAMS,"label_values":[0,1],"engine_steps":0,"training_admitted":false})
    }
}
