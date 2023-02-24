use thiserror::Error as ThisError;

#[derive(ThisError, Debug)]
pub enum Error {
    #[error("TODO: multiple UAPs are not supported yet")]
    MultipleUaps,
    #[error("TODO: BDS is not implemented")]
    BdsNotImplemented,
    #[error("Too many bytes written ({written} >= {limit})")]
    TooManyBytesWritten { written: usize, limit: usize },
    #[error("Too many repetitive items ({count} >= {limit})")]
    TooManyRepetitiveItems { count: usize, limit: usize },
    #[error("Mismatched category: expected {category:03} got {got:03?}")]
    MismatchedCategory { category: u8, got: Option<u64> },
    #[error("CAT{category:03} does not have field `{field}`")]
    InvalidCategoryField { category: u8, field: String },
    #[error("Invalid input: Expected an object for Variable/Fixed/Compound items")]
    ExpectedMap,
    #[error("Repetitive item expects an array")]
    RepetitiveExpectsArray,
    #[error("Invalid input: No subitems were specified in Variable")]
    NoSubitems,
    #[error("Invalid input: Expected number for field `{field}`")]
    ExpectedNumber { field: String },
    #[error("Invalid spec: {child:?}")]
    InvalidSpec {
        #[from]
        child: InvalidSpec,
    },
}

#[derive(ThisError, Debug)]
pub enum InvalidSpec {
    #[error("FRN is not a number in UAP for field `{field}`")]
    FrnNotANumber { field: String },
    #[error("FRN is zero in UAP for field `{field}`")]
    FrnIsZero { field: String },
    #[error("Missing DataItem for field `{field}`")]
    MissingDataItem { field: String },
    #[error("FX bit was used twice")]
    FxUsedTwice,
    #[error("FX bit was used outside of Variable item")]
    FxOutsideVariable,
    #[error("FX field was not used in Variable>Fixed")]
    FxNotUsed,
    #[error("Expected Variable format to contain only Fixed items")]
    ExpectedFixedInVariable,
    #[error("Expected Compound format to start with Variable")]
    ExpectedVariableInCompound,
    #[error("Bad bit/from/to combination: {bit:?}/{from:?}/{to:?}")]
    BadBitCombination {
        bit: Option<u32>,
        from: Option<u32>,
        to: Option<u32>,
    },
    #[error("Bits inside Fixed item must not have gaps, and align with specified length")]
    NonContinousBits,
    #[error("Invalid Compound subitem definition")]
    InvalidCompoundSubitem,
    #[error("Compound subitem BitsPresence is out of bounds")]
    CompoundSubitemOob,
}
