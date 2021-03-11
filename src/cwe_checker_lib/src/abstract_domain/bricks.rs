use std::{
    cmp::{max, min},
    collections::BTreeSet,
};

use super::{AbstractDomain, HasTop};
use crate::prelude::*;
use itertools::Itertools;
/// The Bricks domain that contains a sorted list of single normalized BrickDomains.
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
    /// 2. **merge** successive bricks with the same indices max = 1, min = 1, in a new single brick.
    ///    The new string set is the concatenation of the former two. e.g. B0 = \[{a,cd}\]^{1,1}
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
        // A second vector to do lookups and to iterate over the values.
        let mut lookup = self.unwrap_value();
        let mut unchanged = false;
        while !unchanged {
            for (index, brick_domain) in lookup.iter().enumerate() {
                // Ignore Top value bricks.
                if brick_domain.is_top() {
                    continue;
                }

                // Get the current brick for checks .
                let current_brick = brick_domain.unwrap_value();

                // --Step 1-- Check whether the brick contains the empty string only.
                // If so, remove the brick from the list.
                if current_brick.is_empty_string() {
                    normalized.remove(index);
                    break;
                }

                // --Step 3-- Check whether the lower and upper bound are equal an bigger than 1.
                // If so, create all permutations of the size of min=max and set them to 1.
                if current_brick.min == current_brick.max && current_brick.min > 1 {
                    let transformed_brick = current_brick
                        .transform_brick_with_min_max_equal(current_brick.min as usize);
                    normalized[index] = BrickDomain::Value(transformed_brick);
                    break;
                }

                // --Step 5-- Check whether min >= 1 and max > min.
                // If so, break the brick into simpler bricks.
                if current_brick.min >= 1 && current_brick.max > current_brick.min {
                    let (new_brick1, new_brick2) =
                        current_brick.break_single_brick_into_simpler_bricks();
                    normalized[index] = BrickDomain::Value(new_brick1);
                    normalized.insert(index + 1, BrickDomain::Value(new_brick2));
                    break;
                }

                // Check whether bricks can be merged.
                if let Some(next_brick_domain) = lookup.get(index + 1) {
                    if !next_brick_domain.is_top() {
                        let next_brick = next_brick_domain.unwrap_value();
                        // --Step 2-- Check whether two successive bricks are bound by one in min and max.
                        // If so, merge them by taking the cartesian product of the sequences.
                        if (
                            current_brick.min,
                            current_brick.max,
                            next_brick.min,
                            next_brick.max,
                        ) == (1, 1, 1, 1)
                        {
                            let merged_brick =
                                current_brick.merge_bricks_with_bound_one(next_brick);
                            normalized[index] = BrickDomain::Value(merged_brick);
                            normalized.remove(index + 1);
                            break;
                        }
                        // --Step 4-- Check whether two successive bricks have equal content.
                        // If so, merge them with the same content and add their min and max values together.
                        else if current_brick.sequence == next_brick.sequence {
                            let merged_brick =
                                current_brick.merge_bricks_with_equal_content(next_brick);
                            normalized[index] = BrickDomain::Value(merged_brick);
                            normalized.remove(index + 1);
                            break;
                        }
                    }
                }
            }

            if lookup == normalized {
                unchanged = true;
            } else {
                lookup = normalized.clone();
            }
        }

        BricksDomain::Value(normalized)
    }

    /// Before merging two BrickDomain lists, the shorter one has to be padded
    /// with empty string bricks. To achieve higher positional
    /// correspondence, empty string bricks will be added in a way that
    /// equal bricks have the same indices in both lists.
    fn pad_list(&self, other: &BricksDomain) -> Self {
        let mut short_list = self.unwrap_value();
        let long_list = other.unwrap_value();
        let mut new_list: Vec<BrickDomain> = Vec::new();
        let len_diff = long_list.len() - short_list.len();

        let mut empty_bricks_added = 0;

        for i in 0..long_list.len() {
            if empty_bricks_added >= len_diff {
                new_list.push(short_list.get(0).unwrap().clone());
                short_list.remove(0);
            } else if short_list.is_empty()
                || short_list.get(0).unwrap().unwrap_value()
                    != long_list.get(i).unwrap().unwrap_value()
            {
                new_list.push(BrickDomain::get_empty_brick());
                empty_bricks_added += 1;
            } else {
                new_list.push(short_list.get(0).unwrap().clone());
                short_list.remove(0);
            }
        }

        BricksDomain::Value(new_list)
    }

    /// Unwraps a list of BrickDomains and panic if it's *Top*
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
        if self.is_top() || other.is_top() {
            Self::Top
        } else {
            let self_len = self.unwrap_value().len();
            let other_len = other.unwrap_value().len();
            let mut new_self = self.clone();
            let mut new_other = other.clone();
            if self_len < other_len {
                new_self = self.pad_list(other);
            } else if other_len < self_len {
                new_other = other.pad_list(self);
            }

            let self_list = new_self.unwrap_value();
            let other_list = new_other.unwrap_value();
            let mut merged_list: Vec<BrickDomain> = Vec::new();

            for i in 0..self_list.len() {
                merged_list.push(self_list.get(i).unwrap().merge(other_list.get(i).unwrap()));
            }

            Self::Value(merged_list)
        }
    }

    /// Check if the value is *Top*.
    fn is_top(&self) -> bool {
        matches!(self, Self::Top)
    }
}

impl HasTop for BricksDomain {
    /// Return a *Top* value
    fn top(&self) -> Self {
        Self::Top
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
    /// Returns an empty string brick
    fn get_empty_brick() -> Self {
        BrickDomain::Value(Brick {
            sequence: BTreeSet::new(),
            min: 0,
            max: 0,
        })
    }

    /// Unwraps a brick value and panics if it's *Top*.
    fn unwrap_value(&self) -> Brick {
        match self {
            BrickDomain::Value(brick) => brick.clone(),
            _ => panic!("Unexpected Brick Domain type."),
        }
    }
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
                sequence: self_brick
                    .sequence
                    .union(&other_brick.sequence)
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

/// A single Brick with the set of string, a minimum and maximum bound.
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct Brick {
    sequence: BTreeSet<String>,
    min: u32,
    max: u32,
}

impl Brick {
    /// Checks whether a brick represents an empty string (Rule 1)
    pub fn is_empty_string(&self) -> bool {
        if self.sequence.is_empty() && self.min == 0 && self.max == 0 {
            return true;
        }
        false
    }

    /// **merge** bricks with the same indices max = 1, min = 1, in a new single brick
    /// with the new string set being the concatenation of the former two. e.g. B0 = \[{a,cd}\]^{1,1}
    /// and B1 = \[{b,ef}\]^{1,1} become B_new = \[{ab, aef, cdb, cdef}\]^{1,1}.
    pub fn merge_bricks_with_bound_one(&self, other: Brick) -> Self {
        let product = self
            .sequence
            .iter()
            .cartesian_product(other.sequence.iter())
            .collect_vec();
        let sequence: BTreeSet<String> = product
            .iter()
            .map(|&(str1, str2)| str1.clone() + str2)
            .collect();

        Brick {
            sequence,
            min: 1,
            max: 1,
        }
    }

    /// **transform** a brick in which the number of applications is constant (min = max) into one in which
    /// min = max = 1. e.g. B = \[{a,b}\]^{2,2} => B_new = \[{aa, ab, ba, bb}\]^{1,1}.
    pub fn transform_brick_with_min_max_equal(&self, length: usize) -> Self {
        let permutations: BTreeSet<String> =
            Self::generate_permutations_of_fixed_length(length, &self.sequence, Vec::new())
                .into_iter()
                .collect();
        Brick {
            sequence: permutations,
            min: 1,
            max: 1,
        }
    }

    /// **merge** two bricks in which the set of strings is the same. e.g. B1 = \[S\]^{m1, M1}
    /// and B2 = \[S\]^{m2, M2} => B_new = \[S\]^{m1+m2, M1+M2}
    pub fn merge_bricks_with_equal_content(&self, other: Brick) -> Self {
        Brick {
            sequence: self.sequence.clone(),
            min: self.min + other.min,
            max: self.max + other.max,
        }
    }

    /// **break** a single brick with min >= 1 and max != min into two simpler bricks where B = \[S\]^{min,max} =>
    /// B1 = \[S^min\]^{1,1}, B2 = \[S\]^{0, max-min}.
    /// e.g. B = \[{a}\]^{2,5} => B1 = \[{aa}\]^{1,1}, B2 = \[{a}\]^{0,3}
    pub fn break_single_brick_into_simpler_bricks(&self) -> (Self, Self) {
        let brick_1 = self.transform_brick_with_min_max_equal(self.min as usize);
        let brick_2 = Brick {
            sequence: self.sequence.clone(),
            min: 0,
            max: self.max - self.min,
        };

        (brick_1, brick_2)
    }

    /// Recursive function to generate sequence permutations for a fixed length.
    pub fn generate_permutations_of_fixed_length(
        length: usize,
        sequence: &BTreeSet<String>,
        generated: Vec<String>,
    ) -> Vec<String> {
        let mut new_gen: Vec<String> = Vec::new();
        for s in sequence.iter() {
            if generated.is_empty() {
                new_gen.push(s.to_string());
            } else {
                for g in generated.iter() {
                    new_gen.push(g.clone() + s);
                }
            }
        }

        if new_gen.get(0).unwrap().len() < length {
            return Self::generate_permutations_of_fixed_length(length, sequence, new_gen);
        }

        new_gen
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Setup {
        brick0: BrickDomain,
        brick1: BrickDomain,
        brick2: BrickDomain,
        brick3: BrickDomain,
        brick4: BrickDomain,
        brick5: BrickDomain,
    }

    impl Setup {
        fn new() -> Self {
            Setup {
                brick0: BrickDomain::Value(Setup::mock_brick(
                    vec![String::from("a"), String::from("b")],
                    2,
                    2,
                )),
                brick1: BrickDomain::Value(Setup::mock_brick(
                    vec![String::from("a"), String::from("cd")],
                    1,
                    1,
                )),
                brick2: BrickDomain::Value(Setup::mock_brick(
                    vec![String::from("b"), String::from("ef")],
                    1,
                    1,
                )),
                brick3: BrickDomain::Value(Setup::mock_brick(
                    vec![String::from("a"), String::from("b")],
                    2,
                    3,
                )),
                brick4: BrickDomain::Value(Setup::mock_brick(
                    vec![String::from("a"), String::from("b")],
                    0,
                    1,
                )),
                brick5: BrickDomain::Value(Setup::mock_brick(
                    vec![String::from("a")],
                    1,
                    1,
                )),
            }
        }

        fn mock_brick(sequence: Vec<String>, min: u32, max: u32) -> Brick {
            Brick {
                sequence: sequence.into_iter().collect::<BTreeSet<String>>(),
                min,
                max,
            }
        }
    }

    #[test]
    fn merging_brick_domain() {
        let setup = Setup::new();
        let merged_brick_domain = setup.brick0.merge(&setup.brick4);
        let expected = BrickDomain::Value(
            Setup::mock_brick(vec![String::from("a"), String::from("b")], 0, 2)
        );

        assert_eq!(merged_brick_domain, expected);
    }

    #[test]
    fn merging_bricks_domain() {
        let setup = Setup::new();
        let first_bricks = BricksDomain::Value(vec![setup.brick0.clone()]);
        let second_bricks = BricksDomain::Value(vec![setup.brick0.clone(), setup.brick1.clone()]);

        let merged_bricks = first_bricks.merge(&second_bricks);

        let merged_with_empty = BrickDomain::Value(Setup::mock_brick(
            vec![String::from("a"), String::from("cd")],
            0,
            1,
        ));
        let expected = BricksDomain::Value(vec![setup.brick0.clone(), merged_with_empty]);

        assert_eq!(merged_bricks, expected);
    }

    #[test]
    fn normalizing() {
        let setup = Setup::new();
        let to_normalize: BricksDomain = BricksDomain::Value(vec![setup.brick5, setup.brick3, setup.brick4]); // ["a"]^{1,1}["a", "b"]^{2,3}["a", "b"]^{0,1}
        let normalized = to_normalize.normalize();

        let expected_brick1 = BrickDomain::Value(
            Brick {
                sequence: vec!["aaa", "aab", "aba", "abb"]
                    .iter()
                    .map(|&s| String::from(s))
                    .collect(),
                min: 1,
                max: 1,
            }
        );

        let expected_brick2 = BrickDomain::Value(
            Brick {
                sequence: vec!["a", "b"]
                    .iter()
                    .map(|&s| String::from(s))
                    .collect(),
                min: 0,
                max: 2,
            }
        );

        let expected = BricksDomain::Value(vec![expected_brick1, expected_brick2]);

        assert_eq!(normalized, expected);

    }

    #[test]
    fn generating_permutations_of_fixed_length() {
        let length: usize = 2;
        let sequence: BTreeSet<String> = vec!["a", "b", "c"]
            .into_iter()
            .map(|s| String::from(s))
            .collect();
        let result = Brick::generate_permutations_of_fixed_length(length, &sequence, Vec::new());
        let expected: Vec<String> = vec!["aa", "ba", "ca", "ab", "bb", "cb", "ac", "bc", "cc"]
            .into_iter()
            .map(|s| String::from(s))
            .collect();

        assert_eq!(result, expected);
    }

    #[test]
    fn breaking_single_brick_into_simpler_bricks() {
        let setup = Setup::new();
        let complex_brick = setup.brick3.unwrap_value(); // ["a", "b"]^{2,3}
        let (result1, result2) = complex_brick.break_single_brick_into_simpler_bricks();
        let expected1 = Brick {
            sequence: vec!["aa", "ba", "ab", "bb"]
                .iter()
                .map(|&s| String::from(s))
                .collect(),
            min: 1,
            max: 1,
        };

        let expected2 = Brick {
            sequence: vec!["a", "b"]
                .iter()
                .map(|&s| String::from(s))
                .collect(),
            min: 0,
            max: 1,
        };

        assert_eq!(result1, expected1);
        assert_eq!(result2, expected2);
    }

    #[test]
    fn merging_bricks_with_equal_content() {
        let setup = Setup::new();
        let merge1 = setup.brick0.unwrap_value();
        let merge2 = setup.brick4.unwrap_value();

        let result = merge1.merge_bricks_with_equal_content(merge2);
        let expected = setup.brick3.unwrap_value();

        assert_eq!(result, expected);


    }

    #[test]
    fn transforming_brick_with_min_max_equal() {
        let setup = Setup::new();
        let not_normalized = setup.brick0.unwrap_value();
        let result = not_normalized.transform_brick_with_min_max_equal(not_normalized.min as usize);
        let expected = Brick {
            sequence: vec!["aa", "ba", "ab", "bb"]
                .iter()
                .map(|&s| String::from(s))
                .collect(),
            min: 1,
            max: 1,
        };

        assert_eq!(result, expected);
    }

    #[test]
    fn merging_bricks_with_bound_one() {
        let setup = Setup::new();
        let merge1 = setup.brick1.unwrap_value();
        let merge2 = setup.brick2.unwrap_value();

        let result = merge1.merge_bricks_with_bound_one(merge2);
        let expected = Brick {
            sequence: vec!["ab", "aef", "cdb", "cdef"]
                .iter()
                .map(|&s| String::from(s))
                .collect(),
            min: 1,
            max: 1,
        };

        assert_eq!(result, expected);
    }

    #[test]
    fn empty_string() {
        let setup = Setup::new();
        let brick = setup.brick5.unwrap_value();
        let empty_brick = BrickDomain::get_empty_brick().unwrap_value();

        assert_eq!(brick.is_empty_string(), false);
        assert_eq!(empty_brick.is_empty_string(), true);
    }

    #[test]
    fn padding_list() {
        let setup = Setup::new();
        let empty_brick = BrickDomain::get_empty_brick();
        let short_list = vec![
            setup.brick0.clone(),
            setup.brick1.clone(),
            setup.brick2.clone(),
        ];
        let long_list = vec![
            setup.brick3,
            setup.brick0.clone(),
            setup.brick1.clone(),
            setup.brick4,
            setup.brick5,
        ];

        let new_list = BricksDomain::Value(short_list).pad_list(&BricksDomain::Value(long_list));
        let expected_list = BricksDomain::Value(vec![
            empty_brick.clone(),
            setup.brick0,
            setup.brick1,
            empty_brick.clone(),
            setup.brick2,
        ]);

        assert_eq!(new_list, expected_list);
    }
}
