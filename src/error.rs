use thiserror::Error;

#[derive(Clone, Debug, Eq, Error, Hash, PartialEq)]
pub enum Dq1PasswordError {
    #[error("ゲーム状態が無効: {0}")]
    InvalidGameState(String),

    #[error("復活の呪文の形式が無効: {0}")]
    InvalidPassword(String),

    #[error("CRC 下位バイトが一致しない: expect=0x??{expect:02X}, actual={actual:#04X}")]
    CrcMismatch { expect: u8, actual: u16 },

    #[error("パターンが無効: {0}")]
    InvalidPattern(String),
}

impl Dq1PasswordError {
    pub(crate) fn invalid_game_state(msg: impl Into<String>) -> Self {
        Self::InvalidGameState(msg.into())
    }

    pub(crate) fn invalid_password(msg: impl Into<String>) -> Self {
        Self::InvalidPassword(msg.into())
    }

    pub(crate) fn crc_mismatch(expect: u8, actual: u16) -> Self {
        Self::CrcMismatch { expect, actual }
    }

    pub(crate) fn invalid_pattern(msg: impl Into<String>) -> Self {
        Self::InvalidPattern(msg.into())
    }
}

pub type Dq1PasswordResult<T> = Result<T, Dq1PasswordError>;
