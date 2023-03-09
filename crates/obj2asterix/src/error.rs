use thiserror::Error as ThisError;

#[derive(ThisError, Debug)]
/// Errors encountered while converting to and from asterix.
pub enum Error {
    /// Not yet implemented!
    #[error("TODO: multiple UAPs are not supported yet")]
    MultipleUaps,
    /// Not yet implemented!
    #[error("TODO: BDS is not implemented")]
    BdsNotImplemented,
    /// Amount of bytes written was higher than the expected limit.
    #[error("Too many bytes written ({written} >= {limit})")]
    TooManyBytesWritten {
        /// The amount that was actually written.
        written: usize,
        /// Expected limit.
        limit: usize,
    },
    /// Amount of repetitive items was higher than the expected limit.
    #[error("Too many repetitive items ({count} >= {limit})")]
    TooManyRepetitiveItems {
        /// Amount of repetitive items.
        count: usize,
        /// Expected limit.
        limit: usize,
    },
    /// Category specification doesn't match the category of data.
    #[error("Mismatched category: expected {category:03} got {got:03?}")]
    MismatchedCategory {
        /// Category that was provided.
        category: u8,
        /// Actual category id of the data.
        got: Option<u64>,
    },
    /// Provided category doesn't have expected field.
    #[error("CAT{category:03} does not have field `{field}`")]
    InvalidCategoryField {
        /// Provided category.
        category: u8,
        /// The invalid field.
        field: String,
    },
    /// Provided object is expected to be of type [`serde_json::Map`].
    #[error("Invalid input: Expected an object for Variable/Fixed/Compound items")]
    ExpectedMap,
    /// Provided object is expected to be an array.
    #[error("Invalid input: Expected an array for Variable with one Fixed item")]
    ExpectedArray,
    /// Invalid value provided as an item for a repetitive field.
    #[error("Repetitive item expects an array")]
    RepetitiveExpectsArray,
    /// Variable doesn't contain any subitems.
    #[error("Invalid input: No subitems were specified in Variable")]
    NoVariableSubitems,
    /// Provided object is expected to be a number.
    #[error("Invalid input: Expected number for field `{field}`")]
    ExpectedNumber {
        /// The field that contains a number.
        field: String,
    },
    /// AIS string is too long. Has to be of length less or equal to 8.
    #[error("AIS string is too long: `{string}`")]
    AisTooLong {
        /// Invalid string.
        string: String,
    },
    /// Provided data contains code with length more then 3 bits.
    #[error("Invalid octal code: {code}")]
    InvalidOcatlCode {
        /// The invalid code encountered.
        code: u64,
    },
    /// The field contains invalid ASCII string.
    #[error("Expected string for ASCII encoding for field `{field}`")]
    ExpectedStringForAscii {
        /// The field that caused this error.
        field: String,
    },
    /// String was longer then the expected length.
    #[error("ASCII string is too long `{string}`")]
    AsciiStringTooLong {
        /// The string that caused this error.
        string: String,
    },
    /// The string contains invalid ASCII character.
    #[error("ASCII string `{string}` contains invalid character `{chr}`")]
    InvalidAsciiChar {
        /// The invalid character.
        chr: char,
        /// The string that caused this error.
        string: String,
    },
    /// The AIS string provided contains invalid character.
    #[error("AIS string `{string}` contains invalid character `{chr}`")]
    InvalidAisChar {
        /// The invalid character.
        chr: char,
        /// The string provided.
        string: String,
    },
    /// The AIS code isn't in the expected range.
    #[error("AIS code is invalid `{code}`")]
    InvalidAisCode {
        /// Invalid code.
        code: u8,
    },
    /// The data provided was shorter than the expected length.
    #[error("Reading out of bounds while reading ASTERIX")]
    ReadingOob,
    /// Reader still contains unexpected unread data.
    #[error("Explicit item has some data remaining")]
    ExplicitHasDataLeft,
    /// FSPEC has unknown bits set for provided specification.
    #[error("FSPEC has unknown bits set index={index} byte={byte:08b}")]
    UnknownFspecField {
        /// Byte that contains the invalid bit.
        byte: u8,
        /// Index of the invalid bit.
        index: usize,
    },
    /// Specification parse error.
    #[error("Invalid spec: {child:?}")]
    InvalidSpec {
        /// More info about this error.
        #[from]
        child: InvalidSpec,
    },
}

/// Errors encountered while working with parsed specification.
#[derive(ThisError, Debug)]
pub enum InvalidSpec {
    /// UAP contains invalid FRN for a field. Found a value that can't be parsed to a number.
    #[error("FRN is not a number in UAP for field `{field}`")]
    FrnNotANumber {
        /// The field that contains invalid FRN.
        field: String,
    },
    /// UAP contains invalid FRN for a field. Found zero.
    #[error("FRN is zero in UAP for field `{field}`")]
    FrnIsZero {
        /// The field that contains invalid FRN.
        field: String,
    },
    /// The record doesn't contain a data item.
    #[error("Missing DataItem for field `{field}`")]
    MissingDataItem {
        /// The field data is missing for.
        field: String,
    },
    /// FX bit was used twice.
    #[error("FX bit was used twice")]
    FxUsedTwice,
    /// FX bit is set at the variable end.
    #[error("FX bit was used outside of Variable item")]
    FxOutsideVariable,
    /// FX bit ignored.
    #[error("FX field was not used in Variable>Fixed")]
    FxNotUsed,
    /// Variable doesn't contain fixed items as expected.
    #[error("Expected Variable format to contain only Fixed items")]
    ExpectedFixedInVariable,
    /// Compound is empty or starts with something other than a variable.
    #[error("Expected Compound format to start with Variable")]
    ExpectedVariableInCompound,
    /// Bad bit/from/to combination. Expected only `bit` part or only both `from` and `to` parts.
    #[error("Bad bit/from/to combination: {bit:?}/{from:?}/{to:?}")]
    BadBitCombination {
        /// Bit index. Can't be set together with `from` and `to`.
        bit: Option<u32>,
        /// Bit range start index. Must be set with `to` and can't be set with `bit`.
        from: Option<u32>,
        /// Bit range end index. Must be set with `from` and can't be set with `bit`.
        to: Option<u32>,
    },
    /// `BitReader`/`BitWriter` related error.
    ///
    /// Start doesn't match amount of bits provided at the `BitReader`/`BitWriter` initialization.
    ///
    /// Or doesn't equal to zero at the finalization.
    #[error("Bits inside Fixed item must not have gaps, and align with specified length")]
    NonContinousBits,
    /// Compound index bit not found.
    #[error("Invalid Compound subitem definition")]
    InvalidCompoundSubitem,
    /// Compound subitem index bit is higher than amount of items.
    #[error("Compound subitem BitsPresence is out of bounds")]
    CompoundSubitemOob,
    /// A character is not a multiple of 6 bits.
    #[error("SixBitsChar is not a multiple of six bits")]
    SixBitsCharNotAligned,
    /// A character is not a multiple of 8 bits.
    #[error("ASCII bit length is not a multiple of 8 for field `{field}`")]
    BadAsciiLength {
        /// The field with incorrect character bit length.
        field: String,
    },
    /// Index of expected UAP data was higher than amount of available UAPs.
    #[error("UAP index OOB")]
    UapIndexOob,
}
