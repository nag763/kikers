use sqlx::mysql::MySqlQueryResult;

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
    pub(crate) fn from_expected_affected_rows(
        mysql_result: MySqlQueryResult,
        expected_affected_rows: u64,
    ) -> Self {
        match mysql_result.rows_affected() {
            v if v == expected_affected_rows => Self::Success,
            0 => Self::NoRowsAffected,
            v if v < expected_affected_rows => Self::NotEnoughRowsAffected(v),
            v if expected_affected_rows < v => Self::TooManyRowsAffected(v),
            _ => Self::UnknownResult,
        }
    }
}

impl Into<bool> for TransactionResult {
    fn into(self) -> bool {
        match self {
            TransactionResult::Success => true,
            _ => false,
        }
    }
}
