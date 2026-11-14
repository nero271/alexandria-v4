use crate::error::{AppError, AppResult};
use serde_json::Value;
use std::time::Duration;

pub fn map_err(e: ureq::Error) -> AppError {
    match e {
        ureq::Error::Status(401, _) | ureq::Error::Status(403, _) => {
            AppError::Invalid("Acesso negado pelo serviço. Confira a chave de API.".into())
        }
        ureq::Error::Status(400, _) => {
            AppError::Invalid("Pedido recusado pelo serviço. Confira os dados informados.".into())
        }
        ureq::Error::Status(404, _) => AppError::Invalid("Conteúdo não encontrado no serviço.".into()),
        ureq::Error::Status(code, _) => {
            AppError::Invalid(format!("O serviço respondeu com erro {code}."))
        }
        ureq::Error::Transport(_) => {
            AppError::Invalid("Sem conexão com a internet ou serviço indisponível.".into())
        }
    }
}

pub fn get_json(url: &str, bearer: Option<&str>) -> AppResult<Value> {
    let mut req = ureq::get(url).timeout(Duration::from_secs(15));
    if let Some(key) = bearer {
        req = req.set("Authorization", &format!("Bearer {key}"));
    }
    let resp = req.call().map_err(map_err)?;
    resp.into_json::<Value>()
        .map_err(|e| AppError::Invalid(format!("Resposta inválida do serviço: {e}")))
}
