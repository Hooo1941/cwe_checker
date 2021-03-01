//! The abstract string analysis.
//!
//! The goal of the abstract string analysis is to keep track of all string constants
//! throughout the program in an abstract state that approximates them.

mod context;
use context::*;

mod state;
use state::*;

use crate::{
    utils::log::{CweWarning, LogMessage},
    AnalysisResults,
};

use super::{fixpoint::Computation, forward_interprocedural_fixpoint::GeneralizedContext};

const VERSION: &str = "0.1";

pub static CWE_MODULE: crate::CweModule = crate::CweModule {
    name: "AbstractString",
    version: VERSION,
    run: extract_abstract_string_analysis_results,
};

pub struct AbstractString<'a> {
    computation: Computation<GeneralizedContext<'a, Context<'a>>>,
}

pub fn extract_abstract_string_analysis_results(
    _analysis_results: &AnalysisResults,
    _analysis_params: &serde_json::Value,
) -> (Vec<LogMessage>, Vec<CweWarning>) {
    (Vec::new(), Vec::new())
}
