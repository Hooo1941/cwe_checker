use std::collections::HashMap;

use super::*;
use crate::{abstract_domain::{AbstractDomain, AbstractIdentifier, HasTop}, prelude::*};

/// Contains all information known about the state of a program at a specific point of time.
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct State<T: AbstractDomain + HasTop> {
    known_strings: HashMap<AbstractIdentifier, T>,
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
            known_strings: HashMap::new(),
        }
    }
}
