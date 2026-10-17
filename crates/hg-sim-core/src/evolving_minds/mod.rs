//! Opt-in bounded engineering fixtures. Dynamics are native TOML rewrites.
mod config;
mod output;
mod runtime;
mod source_inventory;
pub use config::*;
pub use output::resume_to;
pub use output::write_smoke;
pub use runtime::{Birth, Event, Frame, Row, Simulation};

mod interventions;
pub use interventions::{probe_checkpoint, write_interventions, Probe};

mod choice;
mod choice_output;
pub use choice::{resume_choice, ChoiceRow, ChoiceSchedule, ChoiceSimulation, Controller, CueMode};
pub use choice_output::write_choice_panel;
mod d0;
pub use d0::{d0_program, D0Instruction, D0Op, D0Program, D0Row, D0Simulation};
mod d0_output;
pub use d0_output::{resume_d0, write_d0_panel};
mod family;
pub use family::{family_program, FamilyBirth, FamilyRow, FamilySimulation, FamilyStep};
mod family_output;
pub use family_output::{resume_family, write_family_panel};

mod mutation;
mod grammar_mutation;
mod mutation_stress;
mod mutation_stress_output;
pub use mutation_stress_output::write_mutation_stress_panel;
pub use mutation_stress::{MutationStressCase, mutation_stress_cases, mutation_stress_registration,mutation_stress_probe_registration,MUTATION_STRESS_PANEL_PROFILE};
pub use grammar_mutation::{grammar_source_candidates, grammar_certificate, GrammarCertificate};
pub use mutation::{source_neighborhood, MutationKernel, MutationRecord, MutationState};

mod mutation_output;
pub use mutation_output::{resume_mutation, write_mutation_panel};

mod lifecycle;
pub use lifecycle::{LifecycleArm, LifecycleState};

mod arbitration;
pub use arbitration::{
    run_arbitration, ArbitrationRecipe, ArbitrationResult, BirthPolicy, OfferTopology,
};

mod lifecycle_output;
pub use lifecycle_output::{resume_lifecycle, write_lifecycle_panel};
mod arbitration_output;
pub use arbitration_output::write_arbitration_panel;

mod admission;
pub use admission::{lottery, AdmissionOffer, AdmissionRecipe, AdmissionRound, AdmissionState};
mod admission_output;
pub use admission_output::{resume_admission, write_admission_panel};

mod stream;
pub use stream::{EvidenceCursor, LiveAncestor, LiveAncestry, TickBundle};
mod stream_output;
pub use stream_output::{
    resume_stream_segment, write_descendant_stream_segment, write_funded_stream_segment,
    write_handoff_stream_segment, write_retained_stream_segment, write_stream_panel,
    write_transfer_stream_segment,
    write_mutation_stress_stream_segment,
    write_stream_segment, SegmentReport, StreamFault, StreamResources,
};

mod neighborhood;
pub use neighborhood::{
    neighborhood_outcome, neighborhood_recipes, ChildLifetime, NeighborhoodOutcome,
    NeighborhoodRecipe,
};
mod neighborhood_output;
pub use neighborhood_output::write_neighborhood_panel;

mod funding;
pub use funding::{
    funding_base_text, funding_recipes, FundingOutcome, FundingRecipe, FUNDING_PROFILE,
};
mod funding_output;
pub use funding_output::write_funding_panel;

mod funded_admission;
pub use funded_admission::{
    funded_admission_cases, FundedAdmissionCase, FundedAdmissionConfig, FundedProgram,
    FUNDED_ADMISSION_PROFILE, FUNDED_STREAM_PROFILE,
};

mod funded_admission_output;
pub use funded_admission_output::write_funded_admission_panel;

mod retained;
pub use retained::{
    retained_cases, RetainedState, RETAINED_POLICY, RETAINED_PROFILE, RETAINED_STREAM_PROFILE,
};

mod retained_output;
pub use retained_output::write_retained_panel;

mod development_output;
pub use development_output::{
    development_cases, write_development_panel, DevelopmentCase, DEVELOPMENT_PROFILE,
};

mod handoff;
pub use handoff::{
    HandoffConfig, SuppliedEdit, HANDOFF_POLICY, HANDOFF_PROFILE, HANDOFF_STREAM_PROFILE,
};
mod handoff_output;
pub use handoff_output::{handoff_cases, write_handoff_panel, HandoffCase, HANDOFF_PANEL_PROFILE};

mod descendant;
pub use descendant::{
    DescendantConfig, DescendantState, DESCENDANT_POLICY, DESCENDANT_PROFILE,
    DESCENDANT_STREAM_PROFILE,
};
mod descendant_output;
pub use descendant_output::{
    descendant_cases, write_descendant_panel, DescendantCase, DESCENDANT_PANEL_PROFILE,
};

mod transfer;
mod transfer_environment;
mod transfer_gate1;
mod transfer_gate1_output;
pub use transfer_gate1_output::write_transfer_gate1;
pub use transfer_gate1::{TransferCase, TransferIntervention, transfer_interventions, transfer_gate1_cases, transfer_gate1_registration, transfer_gate1_evaluation_registration, TRANSFER_GATE1_PROFILE};
pub use transfer_environment::{TemporalPattern, TransferEnvironment, TRANSFER_ENV_PROFILE, TRANSFER_ENV_STREAM_PROFILE,MUTATION_STRESS_PROFILE,MUTATION_STRESS_STREAM_PROFILE,MUTATION_STRESS_POLICY};
pub use transfer::{TransferGate0Config, TRANSFER_GATE0_PROFILE};
mod transfer_output;
pub use transfer_output::write_transfer_gate0;
