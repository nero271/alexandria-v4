use serde::{Serialize, Serializer};

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("erro no banco de dados: {0}")]
    Db(#[from] rusqlite::Error),
    #[error("erro de arquivo: {0}")]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    Invalid(String),
    #[error("jogo não encontrado")]
    NotFound,
    #[error("estado interno indisponível")]
    Lock,
}

// O front-end recebe apenas a mensagem em texto.
impl Serialize for AppError {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&self.to_string())
    }
}

pub type AppResult<T> = Result<T, AppError>;
