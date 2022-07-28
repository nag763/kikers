//! This method is used to verify that a transaction has indeed been executed.
//!
//! It is necesarry when you either do modifications within a MySQL or Mongo
//! database to check whether the modifications have been applied. This class
//! provides a set of simple methods to pass the result of operations.

#[derive(Clone, Debug, PartialEq, Display)]
pub enum TransactionResult {
    /// When the operations ended with success.
    #[display(fmt = "The transaction executed with success")]
    Success,
    /// When no rows have been affected by the modifications.
    #[display(fmt = "No rows have been affected by the transaction")]
    NoRowsAffected,
    /// When too many rows have been affected.
    #[display(fmt = "Too many rows affected : {}", _0)]
    TooManyRowsAffected(u64),
    /// When not enough rows have been affected by the modifications.
    #[display(fmt = "Not enough rows affected : {}", _0)]
    NotEnoughRowsAffected(u64),
    /// When the result is not known, shouldn't be thrown.
    #[display(fmt = "An unexpected result happened")]
    UnknownResult,
}

impl TransactionResult {

    /// Returns the structure from the number of affected rows.
    ///
    /// # Arguments
    /// 
    /// - result : The numbers of rows affected by the updates.
    /// - expected_affected_rows : the expected number of rows by the 
    /// modifications.
    pub(crate) fn from_expected_affected_rows(result: u64, expected_affected_rows: u64) -> Self {
        match result {
            // The first pattern to check is whether it is equal or not,
            // otherwise the no rows affected can be thrown by error.
            v if v == expected_affected_rows => Self::Success,
            0 => Self::NoRowsAffected,
            v if v < expected_affected_rows => Self::NotEnoughRowsAffected(v),
            v if expected_affected_rows < v => Self::TooManyRowsAffected(v),
            _ => Self::UnknownResult,
        }
    }

    /// When a single result is expected from the updates.
    ///
    /// # Arguments
    ///
    /// - result : the number of rows affected by the modifications.
    pub(crate) fn expect_single_result(result: u64) -> Self {
        Self::from_expected_affected_rows(result, 1)
    }
}

#[allow(clippy::from_over_into)]
impl Into<bool> for TransactionResult {
    fn into(self) -> bool {
        matches!(self, TransactionResult::Success)
    }
}
