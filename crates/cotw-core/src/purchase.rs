//! Compra do Premium pelo próprio app.
//!
//! O app gera um código secreto, pede ao servidor de licenças o link de pagamento e, depois que
//! o jogador paga, busca a chave com esse mesmo código e a ativa. O pagamento acontece na página
//! do Mercado Pago (o app nunca vê dados de cartão) e quem assina a chave é o servidor, nunca o
//! app. A rede fica atrás de [`Store`] para os testes não dependerem dela.

use std::fs;
use std::io;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::license::LicenseStatus;

/// Endereço do servidor de licenças, fixado **na hora de compilar** pela variável de ambiente
/// `GREAT_COTW_LICENSE_SERVER` (ex.: `https://licencas.exemplo.com`). Sem ela fica vazio e o
/// botão de compra avisa que a loja ainda não abriu; o código não muda quando o servidor entrar.
pub const LICENSE_SERVER: &str = match option_env!("GREAT_COTW_LICENSE_SERVER") {
    Some(url) => url,
    None => "",
};

/// Quanto tempo o app continua esperando um pagamento (24 horas).
pub const WAIT_SECS: i64 = 86_400;

/// Falhas ao falar com a loja.
#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    /// A loja ainda não foi configurada neste app.
    #[error("a loja ainda não abriu")]
    NotConfigured,
    /// Sem internet ou servidor fora do ar.
    #[error("sem conexão com a loja: {0}")]
    Offline(String),
    /// O servidor recusou o pedido.
    #[error("a loja recusou o pedido")]
    Rejected,
    /// O plano ou o código não têm o formato esperado.
    #[error("pedido de compra inválido")]
    InvalidRequest,
}

impl StoreError {
    /// Código estável para a interface traduzir.
    #[must_use]
    pub fn code(&self) -> &'static str {
        match self {
            Self::NotConfigured => "storeClosed",
            Self::Offline(_) => "storeOffline",
            Self::Rejected => "storeRejected",
            Self::InvalidRequest => "invalidPurchase",
        }
    }
}

/// O que o servidor responde ao buscar a chave.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Claim {
    /// O pagamento ainda não foi confirmado.
    Pending,
    /// Pago: a chave está pronta.
    Ready {
        /// Chave de licença assinada.
        key: String,
        /// Plano comprado.
        plan: String,
    },
}

/// A loja (o servidor de licenças). A versão real fala HTTPS; os testes usam uma falsa.
pub trait Store: Send + Sync {
    /// Cria o pagamento e devolve o link do checkout.
    ///
    /// # Errors
    ///
    /// [`StoreError`] se a loja não estiver configurada, estiver fora do ar ou recusar o pedido.
    fn checkout(&self, plan: &str, token: &str) -> Result<String, StoreError>;

    /// Pergunta se o pagamento desse código já foi confirmado.
    ///
    /// # Errors
    ///
    /// [`StoreError`] se a loja estiver fora do ar ou recusar o pedido.
    fn claim(&self, token: &str) -> Result<Claim, StoreError>;
}

/// Loja que ainda não existe: toda chamada diz que não abriu.
#[derive(Debug, Clone, Copy, Default)]
pub struct ClosedStore;

impl Store for ClosedStore {
    fn checkout(&self, _plan: &str, _token: &str) -> Result<String, StoreError> {
        Err(StoreError::NotConfigured)
    }

    fn claim(&self, _token: &str) -> Result<Claim, StoreError> {
        Err(StoreError::NotConfigured)
    }
}

/// O plano é um dos vendidos?
#[must_use]
pub fn valid_plan(plan: &str) -> bool {
    matches!(plan, "monthly" | "yearly")
}

/// O código tem de ser longo e aleatório (22 a 64 letras, números, `-` e `_`).
#[must_use]
pub fn valid_token(token: &str) -> bool {
    (22..=64).contains(&token.len())
        && token
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
}

/// Uma compra iniciada que ainda espera o pagamento (guardada para sobreviver a fechar o app).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Pending {
    /// Código secreto que busca a chave.
    pub token: String,
    /// Plano escolhido.
    pub plan: String,
    /// Início, em segundos desde 1970.
    pub started: i64,
}

impl Pending {
    /// Lê a compra em espera; arquivo ausente ou corrompido é "nenhuma".
    #[must_use]
    pub fn load(path: &Path) -> Option<Self> {
        let pending: Self = serde_json::from_slice(&fs::read(path).ok()?).ok()?;
        (valid_plan(&pending.plan) && valid_token(&pending.token)).then_some(pending)
    }

    /// Guarda a compra em espera.
    ///
    /// # Errors
    ///
    /// Erro de E/S se o arquivo não puder ser gravado.
    pub fn save(&self, path: &Path) -> io::Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, serde_json::to_vec(self).map_err(io::Error::other)?)
    }

    /// Esquece a compra em espera (não é erro se já não existir).
    pub fn clear(path: &Path) {
        let _ = fs::remove_file(path);
    }

    /// Passou do tempo que vale esperar?
    #[must_use]
    pub fn expired(&self, now: i64) -> bool {
        now - self.started > WAIT_SECS
    }
}

/// Estado da compra para a interface.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PurchaseStatus {
    /// `none` (nada em andamento), `waiting` (esperando o pagamento) ou `activated` (Premium ligado).
    pub state: &'static str,
    /// Plano da compra em andamento ou concluída.
    pub plan: Option<String>,
    /// Situação da licença depois da checagem.
    pub license: LicenseStatus,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plans_and_tokens_must_look_right() {
        assert!(valid_plan("monthly") && valid_plan("yearly"));
        assert!(!valid_plan("weekly") && !valid_plan(""));
        assert!(valid_token(&"a".repeat(22)) && valid_token(&format!("{}-_", "B1".repeat(15))));
        assert!(!valid_token("curto"));
        assert!(!valid_token(&"a".repeat(65)));
        assert!(!valid_token(&format!("{}/", "a".repeat(22))));
        assert!(!valid_token(&format!("{}ç", "a".repeat(22))));
    }

    #[test]
    fn pending_round_trips_and_ignores_garbage() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("sub").join("pending.json");
        assert_eq!(Pending::load(&path), None);
        let pending = Pending {
            token: "a".repeat(24),
            plan: "yearly".into(),
            started: 100,
        };
        pending.save(&path).unwrap();
        assert_eq!(Pending::load(&path), Some(pending.clone()));
        assert!(!pending.expired(100 + WAIT_SECS));
        assert!(pending.expired(100 + WAIT_SECS + 1));
        fs::write(&path, b"{lixo").unwrap();
        assert_eq!(Pending::load(&path), None);
        fs::write(&path, br#"{"token":"x","plan":"yearly","started":1}"#).unwrap();
        assert_eq!(Pending::load(&path), None, "código curto demais");
        Pending::clear(&path);
        Pending::clear(&path);
        assert!(!path.exists());
    }

    #[test]
    fn the_closed_store_says_it_is_closed_and_errors_have_stable_codes() {
        let store = ClosedStore;
        assert_eq!(
            store.checkout("yearly", "x").unwrap_err().code(),
            "storeClosed"
        );
        assert_eq!(store.claim("x").unwrap_err().code(), "storeClosed");
        assert_eq!(StoreError::Offline("x".into()).code(), "storeOffline");
        assert_eq!(StoreError::Rejected.code(), "storeRejected");
        assert_eq!(StoreError::InvalidRequest.code(), "invalidPurchase");
    }
}
