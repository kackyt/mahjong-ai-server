use thiserror::Error;

/// システム（ゲームロジックの実行単位）におけるエラー定義
#[derive(Debug, Error)]
pub enum SystemError {
    /// 不正な操作が行われた場合のエラー。
    /// 例えば、ツモ状態でないのに打牌しようとした場合など。
    #[error("Invalid operation: {0}")]
    InvalidOperation(String),
}
