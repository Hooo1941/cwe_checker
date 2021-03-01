use std::{
    cmp::{max, min},
    collections::HashSet,
};

use super::{AbstractDomain, HasTop};
use crate::prelude::*;
/// The Bricks domain that contains a sorted list of single normalized Brick domains.
/// It represents the composition of a string through sub sequences.
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub enum BricksDomain {
    Top,
    Value(Vec<BrickDomain>),
}

impl BricksDomain {
    /// A set of strings can be built from multiple configurations of bricks
    /// e.g. \[{abc}\]^{1,1} <=> \[{a}\]^{1,1}\[{b}\]^{1,1}\[{c}\]^{1,1}
    ///
    /// Introducing a normalized form \[T\]^{1,1} or \[T\]^{0, max>0}
    /// will keep string representations unambiguous.
    ///
    /// Normalizing can be seen as some kind of fixpoint for a set of 5 rules that are applied
    /// to the list of bricks until the state stays unchanged:
    /// 1. **remove** bricks of the form \[{}\]^{0,0} empty string
    /// 2. **merge** successive bricks with the same indices max = 1, min = 1, in a new single brick
    ///    with the new string set being the concatenation of the former two. e.g. B0 = \[{a,cd}\]^{1,1}
    ///    and B1 = \[{b,ef}\]^{1,1} become B_new = \[{ab, aef, cdb, cdef}\]^{1,1}.
    /// 3. **transform** a brick in which the number of applications is constant (min = max) into one in which
    ///    min = max = 1. e.g. B = \[{a,b}\]^{2,2} => B_new = \[{aa, ab, ba, bb}\]^{1,1}.
    /// 4. **merge** two successive bricks in which the set of strings is the same. e.g. B1 = \[S\]^{m1, M1}
    ///    and B2 = \[S\]^{m2, M2} => B_new = \[S\]^{m1+m2, M1+M2}
    /// 5. **break** a single brick with min >= 1 and max != min into two simpler bricks where B = \[S\]^{min,max} =>
    ///    B1 = \[S^min\]^{1,1}, B2 = \[S\]^{0, max-min}.
    ///    e.g. B = \[{a}\]^{2,5} => B1 = \[{aa}\]^{1,1}, B2 = \[{a}\]^{0,3}
    ///
    /// Since normalization is rather expensive w.r.t. runtime and since it could entail a precision loss,
    /// it is only computed after a merge or widening operation.
    pub fn normalize(&self) -> Self {
        let mut normalized = self.unwrap_value();
        let mut lookup = self.unwrap_value();
        let mut unchanged = false;
        while unchanged {
            for (index, brick_domain) in lookup.iter().enumerate() {
                // Ignore Top value bricks
                if brick_domain.is_top() {
                    continue;
                }
                let brick = brick_domain.unwrap_value();

                // Remove empty string brick
                if BricksDomain::is_empty_string(brick) {
                    normalized.remove(index);
                    continue;
                }

                // Check if bricks can be merged
                if let Some(next_brick_domain) = lookup.get(index + 1) {
                    if next_brick_domain.is_top() {
                        continue;
                    }
                    let next_brick = next_brick_domain.unwrap_value();
                }
            }

            lookup = normalized;
        }

        BricksDomain::Value(normalized)
    }

    /// Checks whether a brick represents an empty string (Rule 1)
    fn is_empty_string(brick: Brick) -> bool {
        if brick.sequences.is_empty() && brick.min == 0 && brick.max == 0 {
            return true;
        }
        false
    }

    /// **merge** successive bricks with the same indices max = 1, min = 1, in a new single brick
    /// with the new string set being the concatenation of the former two. e.g. B0 = \[{a,cd}\]^{1,1}
    /// and B1 = \[{b,ef}\]^{1,1} become B_new = \[{ab, aef, cdb, cdef}\]^{1,1}.
    fn merge_successive_bricks_with_bound_one() -> BrickDomain {}

    /// **transform** a brick in which the number of applications is constant (min = max) into one in which
    /// min = max = 1. e.g. B = \[{a,b}\]^{2,2} => B_new = \[{aa, ab, ba, bb}\]^{1,1}.
    fn transform_brick_with_min_max_equal(brick: Brick) -> BrickDomain {
        if brick.min == brick.max && brick.min > 1 {
            brick.sequences.iter().powerset();
        }

        BrickDomain::Top
    }

    /// **merge** two successive bricks in which the set of strings is the same. e.g. B1 = \[S\]^{m1, M1}
    /// and B2 = \[S\]^{m2, M2} => B_new = \[S\]^{m1+m2, M1+M2}
    fn merge_successive_equal_bricks() -> BrickDomain {}

    /// **break** a single brick with min >= 1 and max != min into two simpler bricks where B = \[S\]^{min,max} =>
    /// B1 = \[S^min\]^{1,1}, B2 = \[S\]^{0, max-min}.
    /// e.g. B = \[{a}\]^{2,5} => B1 = \[{aa}\]^{1,1}, B2 = \[{a}\]^{0,3}
    fn break_single_brick_into_simpler_bricks() -> BrickDomain {}
}

impl BricksDomain {
    fn unwrap_value(&self) -> Vec<BrickDomain> {
        match self {
            BricksDomain::Value(bricks) => bricks.clone(),
            _ => panic!("Unexpected Brick Domain type."),
        }
    }
}

/// Takes care of merging lists of bricks
impl AbstractDomain for BricksDomain {
    fn merge(&self, other: &Self) -> Self {
        todo!()
    }

    fn is_top(&self) -> bool {
        todo!()
    }
}

impl HasTop for BricksDomain {
    fn top(&self) -> Self {
        todo!()
    }
}

/// The single brick domain that represents a set of character sequences
/// as well as the minimum and maximum of the sum of their occurrences.
///
/// e.g. \[{"mo", "de"}\]^{1,2} represents the following set of strings:
/// {mo, de, momo, dede, mode, demo}.
/// The *Top* value represents the powerset over the alphabet
/// of allowed characters with a minimum of 0 and a maximum of positive infinity.
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub enum BrickDomain {
    Top,
    Value(Brick),
}

impl BrickDomain {
    fn unwrap_value(&self) -> Brick {
        match self {
            BrickDomain::Value(brick) => brick.clone(),
            _ => panic!("Unexpected Brick Domain type."),
        }
    }
}

/// A single Brick with the set of string, a minimum and maximum bound.
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct Brick {
    sequences: HashSet<String>,
    min: u32,
    max: u32,
}

/// Takes care of merging single bricks by taking the union
/// of the two brick's string sequences and the minimum and maximum
/// of their respective min and max values.
impl AbstractDomain for BrickDomain {
    fn merge(&self, other: &Self) -> Self {
        if self.is_top() || other.is_top() {
            Self::Top
        } else {
            let self_brick = self.unwrap_value();
            let other_brick = other.unwrap_value();
            Self::Value(Brick {
                sequences: self_brick
                    .sequences
                    .union(&other_brick.sequences)
                    .cloned()
                    .collect(),
                min: min(self_brick.min, other_brick.min),
                max: max(self_brick.max, other_brick.max),
            })
        }
    }

    fn is_top(&self) -> bool {
        matches!(self, Self::Top)
    }
}
