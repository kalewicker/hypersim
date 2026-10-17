//! v14: vacancy-limited funded births and funded parent replacement at capacity.
use super::*;
use crate::economy::funded_fission_bounded_recorded;
pub const PROTOCOL: &str = "selfmod-v14-bounded-v1";
pub const CONTRACT: &str = include_str!("../../../docs/selfmod_v14_bounded_preregistration.md");
pub const LIMIT: usize = 8;

#[path = "birth_replication.rs"]
pub mod replication;
#[path = "birth_financing.rs"]
pub mod financing;
#[path = "birth_durability.rs"]
pub mod durability;
#[path = "birth_variants.rs"]
pub mod variants;
#[path = "birth_second_edit.rs"]
pub mod second_edit;
#[path = "birth_third_edit.rs"]
pub mod third_edit;
#[path = "birth_reversible_edit.rs"]
pub mod reversible_edit;
#[path = "birth_conflict_attribution.rs"]
pub mod conflict_attribution;
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Policy {
    On,
    Off,
    Vacancy8,
    Replace8,
}
pub const POLICIES: [Policy; 4] = [Policy::On, Policy::Off, Policy::Vacancy8, Policy::Replace8];
impl Policy {
    pub fn name(self) -> &'static str {
        match self {
            Self::On => "on",
            Self::Off => "off",
            Self::Vacancy8 => "vacancy8",
            Self::Replace8 => "replace8",
        }
    }
    pub fn parse(s: &str) -> Option<Self> {
        POLICIES.into_iter().find(|p| p.name() == s)
    }
    fn bounded(self) -> bool {
        matches!(self, Self::Vacancy8 | Self::Replace8)
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct Admission {
    parent: u64,
    child: u64,
    population_before: usize,
    retired_tokens: Option<i64>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct Boundary {
    policy: Policy,
    limit: usize,
    population_before: usize,
    eligible: Vec<u64>,
    blocked: Vec<u64>,
    admissions: Vec<Admission>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct Receipt {
    #[serde(flatten)]
    row: Row,
    #[serde(skip_serializing_if = "Option::is_none")]
    bounded: Option<Boundary>,
}
fn finish_phase(s: &mut State, row: Row, policy: Policy) -> Receipt {
    finish_phase_selected(s, row, policy, None, None).0
}
fn finish_phase_selected(s: &mut State, row: Row, policy: Policy, target: Option<u64>, named: Option<(usize, &[u64; 5])>) -> (Receipt, Option<Selection>) {
    let (receipt, selected, _) = finish_phase_targeted(s, row, policy, target, named, None);
    (receipt, selected)
}
// An opt-in adapter for a prospectively specified semantic target. The old
// fixed-parameter hook remains unchanged when semantic_var is absent.
fn finish_phase_targeted(s: &mut State, mut row: Row, policy: Policy, target: Option<u64>, named: Option<(usize, &[u64; 5])>, semantic_var: Option<i64>) -> (Receipt, Option<Selection>, Option<Value>) {
    if !policy.bounded() {
        assert!(target.is_none() && named.is_none() && semantic_var.is_none());
        return (Receipt {
            row: turnover::finish_phase(s, row, policy == Policy::On),
            bounded: None,
        }, None, None);
    }
    let pr = s.cfg.economy_params();
    let before = stocks(s);
    let pre: HashMap<_, _> = before
        .iter()
        .map(|x| (x.id, (x.capital, x.budget)))
        .collect();
    let eligible: Vec<_> = before
        .iter()
        .filter(|x| x.capital >= pr.repro_threshold)
        .map(|x| x.id)
        .collect();
    let mut shadow = s.rng.clone();
    let (out, events) = funded_fission_bounded_recorded(
        &mut s.rs,
        &mut s.rng,
        &pr,
        &s.cfg.caps(),
        &mut s.seq,
        row.absolute_step as i64 - 1,
        LIMIT,
        policy == Policy::Replace8,
        &mut s.led,
    );
    assert_eq!(out.mutant, 0);
    assert_eq!(out.clonal as usize, events.len());
    let mut admissions = Vec::new();
    let mut selected = None;
    let mut targeting = None;
    for event in events {
        let parent = key_u64(event.parent);
        let child = key_u64(event.child);
        let coin = shadow.gen::<f64>();
        let (operator, parameters) = recorded_mutation_draw(&mut shadow);
        let chosen = target == Some(parent);
        if chosen {
            assert!(s.genes[&parent].lineage && s.genes[&parent].member == 0);
            if let Some((op, parameters)) = named {
                let effective;
                let parameters = if let Some(var) = semantic_var {
                    let (resolved, evidence) = second_edit::resolve_parameters(&s.rs, event.child, op, parameters, var);
                    effective = resolved;
                    targeting = Some(evidence);
                    &effective
                } else { parameters };
                assert!(apply_recorded_operator(&mut s.rs, event.child, op, parameters, &s.cfg.caps()).unwrap());
            }
        }
        let gene = register(s, event.child, Some(parent), s.genes[&parent].lineage);
        if chosen {
            selected = Some(Selection { arm: if named.is_some() { Arm::Prune } else { Arm::Faithful }, parent, child, executable: gene.executable, changed_class: gene.class_hash != s.genes[&parent].class_hash, cancelled: false });
        }
        if !chosen || named.is_none() { assert_eq!(gene.class_hash, s.genes[&parent].class_hash); }
        let parent_after_native = pre[&parent].0 - pr.birth_cost;
        assert_eq!(rule_tokens(&s.rs, event.child), pr.birth_cost);
        assert_eq!(
            get_int(&s.rs, event.child, K_CREATION_BUDGET),
            Some(pre[&parent].1)
        );
        if let Some(tokens) = event.retired_tokens {
            assert_eq!(tokens, parent_after_native);
        } else {
            assert_eq!(rule_tokens(&s.rs, event.parent), parent_after_native);
        }
        row.births.push(BirthEvent {
            parent,
            child,
            parent_before: pre[&parent].0,
            parent_after_native,
            child_tokens: pr.birth_cost,
            inherited_budget: pre[&parent].1,
            coin,
            operator,
            parameters,
            cancelled: false,
            gene,
        });
        admissions.push(Admission {
            parent,
            child,
            population_before: event.population_before,
            retired_tokens: event.retired_tokens,
        });
    }
    assert_eq!(shadow.get_word_pos(), s.rng.get_word_pos());
    assert_eq!(probe(&shadow), probe(&s.rng));
    let blocked = eligible
        .iter()
        .filter(|id| !admissions.iter().any(|a| a.parent == **id))
        .copied()
        .collect();
    refill_creation_budgets(&mut s.rs, &pr);
    row.stocks = stocks(s);
    row.ledger = ledger(s);
    row.rng_word_pos = u64::try_from(s.rng.get_word_pos()).unwrap();
    row.rng_probe = probe(&s.rng);
    row.rule_cap_binding = row.stocks.len() >= s.cfg.rule_cap;
    assert!(row.stocks.len() <= LIMIT);
    assert_eq!(row.ledger.audit_residual, 0);
    assert!(target.is_none() || selected.is_some());
    (Receipt {
        row,
        bounded: Some(Boundary {
            policy,
            limit: LIMIT,
            population_before: before.len(),
            eligible,
            blocked,
            admissions,
        }),
    }, selected, targeting)
}
#[derive(Default, Debug, Serialize, Deserialize)]
struct Reproduction {
    births: usize,
    lineage_births: usize,
    retirements: usize,
    retired_tokens: i64,
    late_lineage_retired: i64,
    late_lineage_starved: i64,
    late_birth_windows: [usize; 4],
    late_lineage_births: usize,
    late_start_capital: i64,
    late_min_capital: i64,
    late_max_capital: i64,
    final_capital: i64,
    peak_boundary_population: usize,
    late_start_recorded: bool,
}
fn lineage_capital(s: &State) -> i64 {
    stocks(s)
        .iter()
        .filter(|x| s.genes[&x.id].lineage)
        .map(|x| x.capital)
        .sum()
}
impl Reproduction {
    fn add(&mut self, s: &State, r: &Receipt, horizon: usize) {
        let step = r.row.continuation_step.unwrap();
        let late = step > horizon / 2;
        self.births += r.row.births.len();
        for b in &r.row.births {
            if b.gene.lineage {
                self.lineage_births += 1;
                if late {
                    self.late_lineage_births += 1;
                }
            }
        }
        if late {
            self.late_birth_windows[(step - horizon / 2 - 1) / (horizon / 8)] += r.row.births.len();
            self.late_lineage_starved += r
                .row
                .roots
                .iter()
                .filter(|x| x.lineage && !x.active)
                .map(|x| x.capital_before)
                .sum::<i64>();
        }
        if let Some(bound) = &r.bounded {
            for a in &bound.admissions {
                if let Some(tokens) = a.retired_tokens {
                    self.retirements += 1;
                    self.retired_tokens += tokens;
                    if late && s.genes[&a.parent].lineage {
                        self.late_lineage_retired += tokens;
                    }
                }
            }
        }
        self.final_capital = lineage_capital(s);
        self.peak_boundary_population = self.peak_boundary_population.max(r.row.stocks.len());
        if late {
            self.late_min_capital = self.late_min_capital.min(self.final_capital);
            self.late_max_capital = self.late_max_capital.max(self.final_capital);
        }
    }
}
fn continuation_policy(
    out: &Path,
    mut s: State,
    selected: Selection,
    policy: Policy,
) -> io::Result<Value> {
    let mut file = BufWriter::new(
        OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(out.join("rows.jsonl"))?,
    );
    let mut totals = Totals::new(s.cfg.steps);
    let mut extra = Reproduction::default();
    let mut completed = 0;
    let mut error = None;
    for step in 1..=s.cfg.steps {
        if step == s.cfg.steps / 2 + 1 {
            extra.late_start_capital = lineage_capital(&s);
            extra.late_min_capital = extra.late_start_capital;
            extra.late_max_capital = extra.late_start_capital;
            extra.late_start_recorded = true;
        }
        let mut row = object(&mut s);
        row.continuation_step = Some(step);
        let receipt = finish_phase(&mut s, row, policy);
        totals.add(&receipt.row, &selected, s.cfg.steps);
        extra.add(&s, &receipt, s.cfg.steps);
        serde_json::to_writer(&mut file, &receipt)?;
        file.write_all(b"\n")?;
        file.flush()?;
        completed += 1;
        if s.world.node_count() > s.cfg.node_explosion_cap {
            error = Some("hard node explosion stop".into());
            break;
        }
    }
    let legacy = verdict(&s, &selected, &totals, completed, &error);
    let surplus = totals.late_lineage_income
        - totals.late_lineage_rent
        - extra.late_lineage_retired
        - extra.late_lineage_starved;
    if extra.late_start_recorded {
        assert_eq!(extra.final_capital - extra.late_start_capital, surplus);
    }
    let joint = legacy["bounded_establishment"] == true
        && surplus >= 0
        && extra.late_birth_windows.iter().all(|n| *n > 0)
        && extra.late_lineage_births > 0;
    save(&out.join("final.json"), &snapshot(&s))?;
    save(&out.join("score.json"), &legacy)?;
    save(
        &out.join("reproduction.json"),
        &json!({"joint_success":joint,"retirement_adjusted_late_surplus":surplus,"metrics":extra}),
    )?;
    if error.is_some() {
        return Err(io::Error::other("incomplete continuation; raw retained"));
    }
    Ok(legacy)
}
fn read(p: &Path) -> io::Result<Value> {
    Ok(serde_json::from_slice(&fs::read(p)?)?)
}
fn branch(birth: &Value, out: &Path, policy: Policy) -> io::Result<Value> {
    fs::create_dir(out)?;
    let s = restore(birth["initial"].clone())?;
    let selection = serde_json::from_value(birth["selection"].clone())?;
    assert!(!policy.bounded() || stocks(&s).len() <= LIMIT);
    save(&out.join("birth.json"), birth)?;
    continuation_policy(out, s, selection, policy)
}
pub fn run(out: &Path, preflight: bool) -> io::Result<Value> {
    let _lock = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(out.join("assay.lock"))?;
    let result =
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| run_locked(out, preflight)))
            .unwrap_or_else(|_| Err(io::Error::other("native/harness panic; raw retained")));
    if let Err(e) = &result {
        let prior = read(&out.join("status.json")).ok();
        status(
            out,
            json!({"state":"failed","error":e.to_string(),"last_progress":prior}),
        )?;
    }
    result
}
fn run_locked(out: &Path, preflight: bool) -> io::Result<Value> {
    let seeds: &[u64] = if preflight { &[7, 42] } else { &SEEDS };
    let horizon = if preflight { 64 } else { 2000 };
    let arms: Vec<_> = [Arm::Faithful, Arm::Prune]
        .iter()
        .flat_map(|a| {
            POLICIES
                .iter()
                .map(move |p| format!("{}_{}", a.name(), p.name()))
        })
        .collect();
    save(
        &out.join("manifest.json"),
        &json!({"protocol":PROTOCOL,"engineering_preflight":preflight,"seeds":seeds,"horizon":horizon,"arms":arms,"limit":LIMIT,
        "continuations_requested":seeds.len()*8,"prefix_steps":0,"replication_threshold":6,"future_mutation":false}),
    )?;
    fs::write(out.join("preregistration.md"), CONTRACT)?;
    status(out, json!({"state":"running","continuations_completed":0}))?;
    let mut results = Vec::new();
    for seed in seeds {
        let dir = out.join(format!("seed_{seed}"));
        fs::create_dir(&dir)?;
        for genotype in [Arm::Faithful, Arm::Prune] {
            for policy in POLICIES {
                let arm = format!("{}_{}", genotype.name(), policy.name());
                let control = format!(
                    "{}_{}",
                    genotype.name(),
                    if policy == Policy::Off { "off" } else { "on" }
                );
                let input = out.join(format!("inputs/seed_{seed}/{control}"));
                let birth = read(&input.join("birth.json"))?;
                assert_eq!(birth["initial"]["cfg"]["steps"], horizon);
                let arm_dir = dir.join(&arm);
                let score = branch(&birth, &arm_dir, policy)?;
                assert_eq!(
                    fs::read(arm_dir.join("birth.json"))?,
                    fs::read(input.join("birth.json"))?
                );
                if !policy.bounded() {
                    for file in ["rows.jsonl", "score.json", "final.json"] {
                        assert_eq!(
                            fs::read(arm_dir.join(file))?,
                            fs::read(input.join(file))?,
                            "v13 control diverged: {seed}/{arm}/{file}"
                        );
                    }
                }
                results.push(json!({"seed":seed,"arm":arm,"score":score,"reproduction":read(&arm_dir.join("reproduction.json"))?}));
                status(
                    out,
                    json!({"state":"running","continuations_completed":results.len(),"scientific_advancing_steps":results.len()*horizon,"active_seed":seed}),
                )?;
                println!(
                    "seed {seed} {arm}: {}/{} recorded",
                    results.len(),
                    seeds.len() * 8
                );
                io::stdout().flush()?;
            }
        }
    }
    let summary = json!({"protocol":PROTOCOL,"engineering_preflight":preflight,"continuations_completed":results.len(),"scientific_advancing_steps":results.len()*horizon,
        "new_bounded_steps":seeds.len()*4*horizon,"repeated_control_steps":seeds.len()*4*horizon,"shared_prefix_steps":0,"controls_byte_identical":true,"duplicates_pooled":false,"results":results});
    save(&out.join("summary.json"), &summary)?;
    status(
        out,
        json!({"state":"complete","continuations_completed":results.len(),"scientific_advancing_steps":results.len()*horizon}),
    )?;
    Ok(summary)
}
pub fn replay(birth: &Path, policy: Policy, out: &Path) -> io::Result<Value> {
    branch(&read(birth)?, out, policy)
}
#[cfg(test)]
mod tests {
    use super::*;
    fn funded(n: usize) -> State {
        let mut s = initialize(7, 64);
        while rule_roots(&s.rs).len() < n {
            let root = rule_roots(&s.rs)[0];
            let child = crate::store_ext::duplicate_rule(&mut s.rs, root);
            set_tokens(&mut s.rs, child, 60);
            register(&mut s, child, None, false);
        }
        for r in rule_roots(&s.rs) {
            set_tokens(&mut s.rs, r, 120);
        }
        s.led.cumulative_influx = s.led.free + total_bound(&s.rs);
        s
    }
    #[test]
    fn bounded_births_keep_native_transfer_rng_and_allowance_below_capacity() {
        let mut a = funded(3);
        let mut b = a.clone();
        let row = object(&mut a);
        assert_eq!(row, object(&mut b));
        let native = finish(&mut a, row.clone(), None).0;
        let bounded = finish_phase(&mut b, row, Policy::Vacancy8);
        assert_eq!(native, bounded.row);
        assert_eq!(snapshot(&a), snapshot(&b));
    }
    #[test]
    fn vacancy_blocks_and_replacement_retires_only_funded_parent_at_capacity() {
        let mut s = funded(8);
        let mut vacancy = s.clone();
        let row = object(&mut s);
        assert_eq!(row, object(&mut vacancy));
        let original = stocks(&s);
        let pos = s.rng.get_word_pos();
        let oldseq = s.seq;
        let off = finish_phase(&mut vacancy, row.clone(), Policy::Vacancy8);
        assert!(off.row.births.is_empty());
        assert_eq!(pos, vacancy.rng.get_word_pos());
        let replaced = finish_phase(&mut s, row, Policy::Replace8);
        let bounded = replaced.bounded.unwrap();
        assert_eq!(bounded.admissions.len(), 8);
        assert_eq!(s.seq, oldseq + 8);
        assert_eq!(stocks(&s).len(), 8);
        assert_eq!(s.rng.get_word_pos() - pos, 8 * 14);
        for (a, b) in bounded.admissions.iter().zip(replaced.row.births.iter()) {
            let stock = original.iter().find(|x| x.id == a.parent).unwrap();
            assert_eq!(a.population_before, 8);
            assert_eq!(a.retired_tokens, Some(stock.capital - 50));
            assert_eq!(b.child_tokens, 50);
            assert_eq!(b.inherited_budget, stock.budget);
            assert_eq!(s.genes[&b.child].parent, Some(b.parent));
            assert!(!stocks(&s).iter().any(|x| x.id == b.parent));
        }
        assert_eq!(replaced.row.ledger.audit_residual, 0);
        let mut restored = restore(snapshot(&s)).unwrap();
        for _ in 0..64 {
            let a = object(&mut s);
            let b = object(&mut restored);
            assert_eq!(
                finish_phase(&mut s, a, Policy::Replace8),
                finish_phase(&mut restored, b, Policy::Replace8)
            );
        }
    }
    #[test]
    fn vacancy_admits_only_free_slots_then_replacement_does_not_rebirth_newborns() {
        let mut s = funded(7);
        let row = object(&mut s);
        let r = finish_phase(&mut s, row, Policy::Vacancy8);
        assert_eq!(r.row.births.len(), 1);
        assert_eq!(r.bounded.unwrap().blocked.len(), 6);
        assert_eq!(stocks(&s).len(), 8);
        let mut s = funded(7);
        let row = object(&mut s);
        let r = finish_phase(&mut s, row, Policy::Replace8);
        assert_eq!(r.row.births.len(), 7);
        assert_eq!(
            r.bounded
                .unwrap()
                .admissions
                .iter()
                .filter(|a| a.retired_tokens.is_some())
                .count(),
            6
        );
        assert_eq!(stocks(&s).len(), 8);
    }
}
