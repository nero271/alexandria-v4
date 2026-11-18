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

/// Cliente que não segue redirecionamentos sozinho (quem decide é `get_https`).
pub fn https_agent() -> ureq::Agent {
    ureq::AgentBuilder::new()
        .redirects(0)
        .timeout_connect(Duration::from_secs(20))
        .timeout_read(Duration::from_secs(45))
        .user_agent("ALEXANDRIA")
        .build()
}

fn resolve_location(base: &str, loc: &str) -> AppResult<String> {
    let loc = loc.trim();
    if loc.starts_with("https://") {
        return Ok(loc.to_string());
    }
    if loc.starts_with('/') {
        if let Some(auth) = crate::safe::authority(base) {
            return Ok(format!("https://{auth}{loc}"));
        }
    }
    Err(AppError::Invalid("O servidor redirecionou para um endereço não seguro (só https é aceito).".into()))
}

/// GET que segue até 5 redirecionamentos, sempre exigindo https. Com `range_from`, pede só
/// a partir desse byte. A resposta 416 (pedaço inválido) é devolvida para quem chamou decidir.
pub fn get_https(agent: &ureq::Agent, url: &str, range_from: Option<u64>) -> AppResult<ureq::Response> {
    let mut current = url.to_string();
    for _ in 0..6 {
        if !crate::safe::is_https_url(&current) {
            return Err(AppError::Invalid("O endereço precisa começar com https://".into()));
        }
        let mut req = agent.get(&current);
        if let Some(n) = range_from {
            req = req.set("Range", &format!("bytes={n}-"));
        }
        let resp = match req.call() {
            Ok(r) => r,
            Err(ureq::Error::Status(416, r)) => r,
            Err(e) => return Err(map_err(e)),
        };
        let status = resp.status();
        if (300..400).contains(&status) && status != 304 {
            let loc = resp
                .header("Location")
                .ok_or_else(|| AppError::Invalid("O servidor mandou um redirecionamento sem destino.".into()))?
                .to_string();
            current = resolve_location(&current, &loc)?;
            continue;
        }
        return Ok(resp);
    }
    Err(AppError::Invalid("O servidor redirecionou vezes demais.".into()))
}

pub fn get_json_headers(url: &str, headers: &[(&str, &str)]) -> AppResult<Value> {
    let mut req = ureq::get(url).timeout(Duration::from_secs(15));
    for (k, v) in headers {
        req = req.set(k, v);
    }
    let resp = req.call().map_err(map_err)?;
    resp.into_json::<Value>()
        .map_err(|e| AppError::Invalid(format!("Resposta inválida do serviço: {e}")))
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
