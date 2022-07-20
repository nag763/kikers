#[derive(Clone, Debug, PartialEq, Display)]
pub enum TransactionResult {
    #[display(fmt = "The transaction executed with success")]
    Success,
    #[display(fmt = "No rows have been affected by the transaction")]
    NoRowsAffected,
    #[display(fmt = "Too many rows affected : {}", _0)]
    TooManyRowsAffected(u64),
    #[display(fmt = "Not enough rows affected : {}", _0)]
    NotEnoughRowsAffected(u64),
    #[display(fmt = "An unexpected result happened")]
    UnknownResult,
}

impl TransactionResult {
    pub(crate) fn from_expected_affected_rows(result: u64, expected_affected_rows: u64) -> Self {
        match result {
            v if v == expected_affected_rows => Self::Success,
            0 => Self::NoRowsAffected,
            v if v < expected_affected_rows => Self::NotEnoughRowsAffected(v),
            v if expected_affected_rows < v => Self::TooManyRowsAffected(v),
            _ => Self::UnknownResult,
        }
    }

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
