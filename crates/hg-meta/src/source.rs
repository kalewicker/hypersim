use crate::{read_program, CandidateRule, MetaLimits, RuleProgram, L_RULE_ROOT};
use hg_core::{GraphSnapshot, NodeId};
use hg_rules::{EpochPolicy, RuleCompileReport, RuleSetDelta, RuleSource};
use std::collections::HashMap;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct GraphRuleSource {
    pub limits: MetaLimits,
    pub epoch: EpochPolicy,
    pub meta_visible: bool,
    pub anchors: Vec<NodeId>,
    /// Seeded meta editors are explicit static motifs, not counted as novel
    /// endogenous rules. They use the same engine matcher/apply/conflict path.
    pub editors: Vec<hg_rules::CompiledRule>,
    pub vocabulary_enabled: bool,
    pub witness_repair_enabled: bool,
    pub sealed_repair_enabled: bool,
    pub complete_witness_enabled: bool,
    previous: HashMap<NodeId, Vec<u8>>,
    table: Vec<Vec<u8>>,
    initialized: bool,
}
impl GraphRuleSource {
    pub fn new(epoch: EpochPolicy, limits: MetaLimits, meta_visible: bool) -> Self {
        Self {
            limits,
            epoch,
            meta_visible,
            anchors: Vec::new(),
            editors: Vec::new(),
            vocabulary_enabled: false,
            witness_repair_enabled: false,
            sealed_repair_enabled: false,
            complete_witness_enabled: false,
            previous: HashMap::new(),
            table: Vec::new(),
            initialized: false,
        }
    }
}
impl RuleSource for GraphRuleSource {
    fn checkpoint(&self) -> Result<hg_rules::source::RuleSourceCheckpoint, String> {
        Ok(hg_rules::source::RuleSourceCheckpoint {
            kind: "GraphRuleSource-v1".into(),
            payload: bincode::serialize(self).map_err(|e| e.to_string())?,
        })
    }
    fn epoch_len(&self) -> EpochPolicy {
        self.epoch
    }
    fn rules(&mut self, s: &GraphSnapshot<'_>, step: u64) -> RuleSetDelta {
        if self.initialized && (self.epoch == EpochPolicy::Never || !self.epoch.due(step)) {
            return RuleSetDelta::Unchanged;
        }
        let roots: Vec<_> = s
            .nodes()
            .filter(|(_, n)| n.label.0 == L_RULE_ROOT)
            .map(|(id, _)| id)
            .collect();
        if !self.initialized {
            self.anchors = roots.clone();
        }
        let mut report = RuleCompileReport {
            step,
            ..RuleCompileReport::default()
        };
        let mut decoded: Vec<(NodeId, RuleProgram)> = Vec::new();
        for root in roots {
            let is_complete = s.get_node(root).and_then(|n| n.get_property(crate::K_ENCODING))
                == Some(hg_core::PropertyValue::Int(crate::COMPLETE_VERSION));
            let result = if is_complete != self.complete_witness_enabled {
                Err(crate::Validator::Compilability)
            } else { read_program(s, root, &self.limits) };
            match result {
                Ok(c) => decoded.push((root, c)),
                Err(v) => {
                    report.quarantined += 1;
                    report.quarantine_by_validator[v as usize] += 1;
                }
            }
        }
        // Full bytes, not a potentially-colliding or incomplete hash. Identical
        // executable clones are interchangeable; no allocation/history tie key.
        let mut current = HashMap::new();
        let mut flat: Vec<(Vec<u8>, Vec<u8>, CandidateRule, hg_rules::CompiledRule)> = Vec::new();
        let mut table = Vec::new();
        for (root, p) in decoded {
            let bytes = p.execution_bytes();
            match p.compile(&self.limits, self.meta_visible) {
                Ok(compiled) => {
                    report.validated_regions += 1;
                    current.insert(root, bytes);
                    for (c, r) in p.candidates().into_iter().zip(compiled) {
                        flat.push((c.rcf_bytes(), c.execution_bytes(), c, r));
                    }
                }
                Err(v) => {
                    report.quarantined += 1;
                    report.quarantine_by_validator[v as usize] += 1;
                }
            }
        }
        flat.sort_by(|a, b| (&a.0, &a.1).cmp(&(&b.0, &b.1)));
        let mut rules = Vec::new();
        for (_, bytes, c, r) in flat {
            report.classes.push(c.rcf_hash());
            table.push(bytes);
            rules.push(r);
        }
        if self.vocabulary_enabled {
            let read_instruction = if self.complete_witness_enabled {crate::read_complete_instruction} else {crate::read_instruction};
            let compile_operations = if self.complete_witness_enabled {crate::compile_complete_operations} else {crate::compile_operations};
            let execution_bytes = |op: crate::MetaOperation| if self.complete_witness_enabled {crate::complete_operation_bytes(op)} else {op.execution_bytes()};
            let mut operations = std::collections::BTreeSet::new();
            for (root, _) in s
                .nodes()
                .filter(|(_, n)| n.label.0 == crate::L_META_INSTRUCTION)
            {
                match read_instruction(s, root).and_then(|instruction| {
                    if !instruction.operation.admitted(self.witness_repair_enabled, self.sealed_repair_enabled) {
                        return Err(crate::Validator::Compilability);
                    }
                    compile_operations(instruction.operation, &self.limits)?;
                    Ok(instruction)
                }) {
                    Ok(instruction) => {
                        report.validated_regions += 1;
                        current.insert(root, execution_bytes(instruction.operation));
                        operations.insert(instruction.operation);
                    }
                    Err(v) => {
                        report.quarantined += 1;
                        report.quarantine_by_validator[v as usize] += 1;
                    }
                }
            }
            for op in operations {
                for (variant,rule) in compile_operations(op, &self.limits).unwrap().into_iter().enumerate() {
                    let mut bytes=execution_bytes(op);
                    if op.code()>=3 {bytes.extend((variant as u32).to_le_bytes());}
                    table.push(bytes);
                    report.encoded_meta_operations.push(op.name().into());
                    rules.push(rule);
                }
            }
        }
        report.installed = current
            .iter()
            .filter(|(id, bytes)| self.previous.get(id) != Some(bytes))
            .count();
        report.retired = self
            .previous
            .iter()
            .filter(|(id, bytes)| current.get(id) != Some(bytes))
            .count();
        report.changed = !self.initialized || self.table != table;
        self.previous = current;
        self.table = table;
        self.initialized = true;
        rules.extend(self.editors.iter().cloned());
        RuleSetDelta::Replace { rules, report }
    }
}
