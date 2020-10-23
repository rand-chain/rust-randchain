use compact::Compact;
use storage::Error as DBError;

#[derive(Debug, PartialEq)]
/// All possible verification errors
pub enum Error {
    /// has an equal duplicate in the chain
    Duplicate,
    /// Invalid proof-of-work (Block hash does not satisfy nBits)
    Pow,
    /// Futuristic timestamp
    FuturisticTimestamp,
    /// Invalid timestamp
    Timestamp,
    /// nBits do not match difficulty rules
    Difficulty { expected: Compact, actual: Compact },
    /// Block transactions are not final.
    NonFinalBlock,
    /// Old version block.
    OldVersionBlock,
    /// SegWit: bad witess nonce size
    WitnessInvalidNonceSize,
    /// Database error
    Database(DBError),
}

impl From<DBError> for Error {
    fn from(err: DBError) -> Self {
        Error::Database(err)
    }
}
