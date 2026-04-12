use std::fmt;
use thiserror::Error;

/// 不正操作の理由を表す Newtype
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct InvalidOperationReason(pub String);

impl fmt::Display for InvalidOperationReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// システム（ゲームロジックの実行単位）におけるエラー定義
#[derive(Debug, Error)]
pub enum SystemError {
    /// 不正な操作が行われた場合のエラー。
    /// 例えば、ツモ状態でないのに打牌しようとした場合など。
    #[error("Invalid operation: {0}")]
    InvalidOperation(InvalidOperationReason),
}
