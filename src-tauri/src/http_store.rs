//! A loja de verdade: fala HTTPS com o servidor de licenças (repositório `great-cotw-one-platform`).
//!
//! A conversa é pequena: pedir o link de pagamento e perguntar se o pagamento já foi
//! confirmado. Cartão e Pix passam só pela página do Mercado Pago, nunca pelo app.

use std::time::Duration;

use cotw_core::purchase::{Claim, Store, StoreError};
use serde_json::{Value, json};

/// Cliente do servidor de licenças.
pub struct HttpStore {
    base: String,
    agent: ureq::Agent,
}

impl HttpStore {
    /// `base` é o endereço do servidor (`https://licencas.exemplo.com`); a barra final é ignorada.
    #[must_use]
    pub fn new(base: &str) -> Self {
        // Só HTTPS, para nunca mandar a chave de ativação em texto aberto; servidor local de teste
        // (`http://localhost` ou `127.0.0.1`) é a única exceção.
        let local = base.starts_with("http://localhost") || base.starts_with("http://127.0.0.1");
        let agent = ureq::Agent::config_builder()
            .timeout_global(Some(Duration::from_secs(20)))
            .https_only(!local)
            .build()
            .into();
        Self {
            base: base.trim_end_matches('/').to_owned(),
            agent,
        }
    }
}

fn failure(error: &ureq::Error) -> StoreError {
    match error {
        ureq::Error::StatusCode(code) if (400..500).contains(code) => StoreError::Rejected,
        other => StoreError::Offline(other.to_string()),
    }
}

/// Link do checkout vindo da resposta; só aceita `https://`, para o app nunca abrir outra coisa.
fn parse_checkout(body: &Value) -> Result<String, StoreError> {
    body.get("url")
        .and_then(Value::as_str)
        .filter(|url| url.starts_with("https://"))
        .map(str::to_owned)
        .ok_or(StoreError::Rejected)
}

fn parse_claim(body: &Value) -> Result<Claim, StoreError> {
    match body.get("status").and_then(Value::as_str) {
        Some("pending") => Ok(Claim::Pending),
        Some("ready") => {
            let key = body.get("key").and_then(Value::as_str);
            let plan = body.get("plan").and_then(Value::as_str);
            match (key, plan) {
                (Some(key), Some(plan)) => Ok(Claim::Ready {
                    key: key.to_owned(),
                    plan: plan.to_owned(),
                }),
                _ => Err(StoreError::Rejected),
            }
        }
        _ => Err(StoreError::Rejected),
    }
}

impl Store for HttpStore {
    fn checkout(&self, plan: &str, token: &str) -> Result<String, StoreError> {
        let mut response = self
            .agent
            .post(&format!("{}/v1/checkout", self.base))
            .send_json(json!({ "plan": plan, "token": token }))
            .map_err(|e| failure(&e))?;
        let body: Value = response.body_mut().read_json().map_err(|e| failure(&e))?;
        parse_checkout(&body)
    }

    fn claim(&self, token: &str) -> Result<Claim, StoreError> {
        let mut response = self
            .agent
            .get(&format!("{}/v1/claim/{token}", self.base))
            .call()
            .map_err(|e| failure(&e))?;
        let body: Value = response.body_mut().read_json().map_err(|e| failure(&e))?;
        parse_claim(&body)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_https_checkout_links_are_accepted() {
        let ok = parse_checkout(&json!({ "url": "https://mp.example/x" })).unwrap();
        assert_eq!(ok, "https://mp.example/x");
        for bad in [
            json!({ "url": "http://mp.example/x" }),
            json!({ "url": "file:///c:/windows" }),
            json!({ "url": 3 }),
            json!({}),
        ] {
            assert!(matches!(parse_checkout(&bad), Err(StoreError::Rejected)));
        }
    }

    #[test]
    fn claim_answers_are_understood_and_anything_else_is_refused() {
        assert_eq!(
            parse_claim(&json!({ "status": "pending" })).unwrap(),
            Claim::Pending
        );
        let ready = json!({ "status": "ready", "key": "GCO1.a.b", "plan": "yearly" });
        assert_eq!(
            parse_claim(&ready).unwrap(),
            Claim::Ready {
                key: "GCO1.a.b".into(),
                plan: "yearly".into()
            }
        );
        for bad in [
            json!({ "status": "ready", "key": "GCO1.a.b" }),
            json!({ "status": "ready" }),
            json!({ "status": "unknown" }),
            json!({}),
        ] {
            assert!(matches!(parse_claim(&bad), Err(StoreError::Rejected)));
        }
    }

    #[test]
    fn a_plain_http_server_that_is_not_local_is_never_contacted() {
        let store = HttpStore::new("http://licencas.exemplo.com");
        assert!(matches!(store.claim("x"), Err(StoreError::Offline(_))));
        assert!(matches!(
            store.checkout("monthly", "x"),
            Err(StoreError::Offline(_))
        ));
    }

    #[test]
    fn a_dead_server_is_offline_and_a_4xx_is_a_refusal() {
        // Porta fechada: ninguém escuta em 127.0.0.1:9.
        let store = HttpStore::new("http://127.0.0.1:9/");
        assert!(matches!(store.claim("x"), Err(StoreError::Offline(_))));
        assert!(matches!(
            failure(&ureq::Error::StatusCode(404)),
            StoreError::Rejected
        ));
        assert!(matches!(
            failure(&ureq::Error::StatusCode(502)),
            StoreError::Offline(_)
        ));
    }
}
