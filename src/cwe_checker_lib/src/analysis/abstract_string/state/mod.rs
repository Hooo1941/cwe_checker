use std::collections::BTreeMap;

use super::*;
use crate::{
    abstract_domain::{AbstractDomain, HasTop},
    intermediate_representation::Variable,
    prelude::*,
};

/// Contains all information known about the state of a program at a specific point of time.
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct State<T: AbstractDomain + HasTop> {
    /// Maps a register variable to an abstract string domain value.
    /// All registers not contained, do not hold a string value.
    /// This represents the rare cases where a string constant is directly
    /// put into the register.
    register: BTreeMap<Variable, T>,
    /// Maps a pointer plus offset to an abstract string domain value in memory.
    pointer: BTreeMap<(Variable, i64), T>,
}

impl<T: AbstractDomain + HasTop> AbstractDomain for State<T> {
    fn merge(&self, other: &Self) -> Self {
        todo!()
    }

    fn is_top(&self) -> bool {
        todo!()
    }
}

impl<T: AbstractDomain + HasTop> State<T> {
    pub fn new() -> State<T> {
        State {
            register: BTreeMap::new(),
            pointer: BTreeMap::new(),
        }
    }
}
