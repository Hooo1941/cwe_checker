use std::cmp::{max, min};

use super::{AbstractDomain, BitvectorDomain, HasByteSize, HasTop, RegisterDomain};
use crate::{intermediate_representation::{BinOpType, CastOpType, UnOpType}, prelude::*};

/// The `StringLengthDomain` is a abstract domain that describes the lower and upper bound
/// of the size of a string at a certain point in time.
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Hash, Clone)]
pub enum StringLengthDomain {
    Top(ByteSize),
    Value((ByteSize, ByteSize)),
}

impl AbstractDomain for StringLengthDomain {
    /// merge two string lengths values. Returns *Top* if either of them is *Top*,
    /// in case they are the same, the first value is returned, and finally,
    /// a new value is returned with the mininum lower bound and maximum upper bound
    /// in case they are different.
    fn merge(&self, other: &Self) -> Self {
        use StringLengthDomain::*;
        match self {
            Value(bounds) => match other {
                Value(other_bounds) => {
                    if bounds == other_bounds {
                        return self.clone();
                    } else {
                        Value(
                            min(bounds.0, other_bounds.0),
                            max(bounds.1, other_bounds.1),
                        )
                    }
                }
                Top(_) => other.top(),
            },
            Top(_) => self.top(),
        }
    }

    /// Check if the value is *Top*
    fn is_top(&self) -> bool {
        matches!(self, Self::Top(_))
    }
}

impl HasTop for StringLengthDomain {
    fn top(&self) -> StringLengthDomain {
        StringLengthDomain::Top(self.bytesize())
    }
}

impl HasByteSize for StringLengthDomain {
    fn bytesize(&self) -> ByteSize {
        use BitvectorDomain::*;
        match self {
            Top(bytesize) => *bytesize,
            // Return the max possible string length
            Value(bounds) => bounds.1,
        }
    }
}

impl RegisterDomain for StringLengthDomain {
    fn new_top(bytesize: ByteSize) -> StringLengthDomain {
        StringLengthDomain::Top(bytesize)
    }

    fn bin_op(&self, op: BinOpType, rhs: &Self) -> Self {
        use BinOpType::*;
        use StringLengthDomain::*;
        match (self, op, rhs) {
        }
    }

    fn un_op(&self, op: UnOpType) -> Self {
        
    }

    fn subpiece(&self, low_byte: ByteSize, size: ByteSize) -> Self {
        
    }

    fn cast(&self, kind: CastOpType, width: ByteSize) -> Self {
        
    }
}
