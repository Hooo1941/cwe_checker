use crate::{
    analysis::graph::Graph,
    intermediate_representation::{Project, Tid},
    utils::binary::RuntimeMemoryImage,
};

use std::collections::HashSet;

// contains trait implementations for the `Context` struct,
// especially the implementation of the `interprocedural_fixpoint::Context` trait.
mod trait_impls;

pub struct Context<'a> {
    /// The program control flow graph on which the fixpoint will be computed
    pub graph: Graph<'a>,
    /// A reference to the `Project` object representing the binary
    pub project: &'a Project,
    /// The runtime memory image for reading global read-only variables.
    /// Note that values of writeable global memory segments are not tracked.
    pub runtime_memory_image: &'a RuntimeMemoryImage,
}

impl<'a> Context<'a> {
    pub fn new(project: &'a Project, runtime_memory_image: &'a RuntimeMemoryImage) -> Context<'a> {
        let extern_symbol_tid_set: HashSet<Tid> = project
            .program
            .term
            .extern_symbols
            .iter()
            .map(|symb| symb.tid.clone())
            .collect();
        let graph =
            crate::analysis::graph::get_program_cfg(&project.program, extern_symbol_tid_set);
        Context {
            graph,
            project,
            runtime_memory_image,
        }
    }
}
