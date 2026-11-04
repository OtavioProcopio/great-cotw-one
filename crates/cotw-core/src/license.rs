//! Licença Premium: chave anual assinada, verificada sem internet.
//!
//! Premium = gravar alterações no jogo (liberar zonas e editar animais que já existem). Gratuito =
//! ver o mapa, os animais, o perfil, as missões e fazer backups.
//!
//! A chave é um texto `GCO1.<dados>.<assinatura>` (base64 sem preenchimento). Os dados são um JSON
//! com o produto, o plano (mensal ou anual), o e-mail do comprador e as datas de emissão e vencimento;
//! a assinatura é Ed25519.
//! A chave pública vem embutida no app (`license_public.key`); a privada fica só com o vendedor,
//! fora do repositório, e é usada pela ferramenta `gco-license` para emitir chaves.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use base64::Engine as _;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use chrono::Utc;
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};

/// Nome do produto dentro da chave (impede usar chaves de outros produtos).
pub const PRODUCT: &str = "great-cotw-one";
/// Versão do formato da chave.
const FORMAT_VERSION: u8 = 1;
/// Começo de toda chave.
const PREFIX: &str = "GCO1";
/// Segundos em um dia.
const DAY: i64 = 86_400;
/// Quantos dias antes do vencimento o app começa a avisar.
pub const WARN_DAYS: i64 = 30;
/// Chave pública do vendedor, embutida no app.
const EMBEDDED_PUBLIC_KEY: [u8; 32] = *include_bytes!("license_public.key");

/// Quem decide se a licença Premium está ativa.
pub trait LicenseCheck: Send + Sync {
    /// `true` se o usuário tem licença Premium ativa.
    fn is_premium(&self) -> bool;

    /// Situação completa para a interface.
    fn status(&self) -> LicenseStatus {
        if self.is_premium() {
            LicenseStatus::active_without_details()
        } else {
            LicenseStatus::free()
        }
    }
}

/// Sem licença: tudo o que é Premium fica bloqueado.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NoLicense;

impl LicenseCheck for NoLicense {
    fn is_premium(&self) -> bool {
        false
    }
}

/// Licença de valor fixo (testes e desenvolvimento).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FixedLicense(pub bool);

impl LicenseCheck for FixedLicense {
    fn is_premium(&self) -> bool {
        self.0
    }
}

/// Em que pé está a licença.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum LicenseState {
    /// Sem chave: modo gratuito.
    Free,
    /// Chave válida.
    Active,
    /// Chave válida, mas vence em 30 dias ou menos.
    Expiring,
    /// Chave que já venceu: modo gratuito.
    Expired,
}

/// Situação da licença para a interface.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LicenseStatus {
    /// As funções Premium estão liberadas.
    pub premium: bool,
    /// Situação.
    pub state: LicenseState,
    /// E-mail do comprador.
    pub email: Option<String>,
    /// Vencimento, em segundos desde 1970.
    pub expires: Option<i64>,
    /// Dias que faltam (0 no dia do vencimento).
    pub days_left: Option<i64>,
    /// Plano da chave (`monthly` ou `yearly`); `None` se a chave não diz.
    pub plan: Option<String>,
}

impl LicenseStatus {
    /// Modo gratuito.
    #[must_use]
    pub fn free() -> Self {
        Self {
            premium: false,
            state: LicenseState::Free,
            email: None,
            expires: None,
            days_left: None,
            plan: None,
        }
    }

    fn active_without_details() -> Self {
        Self {
            premium: true,
            state: LicenseState::Active,
            email: None,
            expires: None,
            days_left: None,
            plan: None,
        }
    }
}

/// Erros ao ler ou verificar uma chave.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum LicenseError {
    /// O texto não tem o formato de uma chave.
    #[error("a chave não tem o formato esperado")]
    Format,
    /// A assinatura não confere: chave alterada ou de outro vendedor.
    #[error("a assinatura da chave não confere")]
    Signature,
    /// A chave é de outro produto ou de uma versão que este app não conhece.
    #[error("a chave não é deste produto")]
    Product,
    /// A chave já venceu.
    #[error("a chave já venceu")]
    Expired,
}

impl LicenseError {
    /// Código estável para a interface traduzir.
    #[must_use]
    pub fn code(self) -> &'static str {
        match self {
            Self::Format => "licenseFormat",
            Self::Signature => "licenseSignature",
            Self::Product => "licenseProduct",
            Self::Expired => "licenseExpired",
        }
    }
}

#[derive(Serialize, Deserialize)]
struct Payload {
    v: u8,
    p: String,
    /// Plano: `monthly` ou `yearly` (chaves antigas ou manuais podem não ter).
    #[serde(default)]
    l: String,
    e: String,
    i: i64,
    x: i64,
}

/// O que a chave diz.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LicenseInfo {
    /// E-mail do comprador.
    pub email: String,
    /// Plano (`monthly`, `yearly` ou vazio).
    pub plan: String,
    /// Emissão, em segundos desde 1970.
    pub issued: i64,
    /// Vencimento, em segundos desde 1970.
    pub expires: i64,
}

impl LicenseInfo {
    /// Dias que faltam para vencer (negativo se já venceu).
    #[must_use]
    pub fn days_left(&self, now: i64) -> i64 {
        (self.expires - now).div_euclid(DAY) + i64::from((self.expires - now).rem_euclid(DAY) > 0)
    }

    /// Situação da chave num instante.
    #[must_use]
    pub fn status_at(&self, now: i64) -> LicenseStatus {
        let days = self.days_left(now);
        let (premium, state) = if now >= self.expires {
            (false, LicenseState::Expired)
        } else if days <= WARN_DAYS {
            (true, LicenseState::Expiring)
        } else {
            (true, LicenseState::Active)
        };
        LicenseStatus {
            premium,
            state,
            email: Some(self.email.clone()),
            expires: Some(self.expires),
            days_left: Some(days.max(0)),
            plan: (!self.plan.is_empty()).then(|| self.plan.clone()),
        }
    }
}

/// Dias de validade do plano mensal (um mês e uma folga de um dia).
pub const MONTHLY_DAYS: i64 = 31;
/// Dias de validade do plano anual (um ano e uma folga de um dia).
pub const YEARLY_DAYS: i64 = 366;

/// Emite uma chave sem plano definido (testes e casos manuais).
#[must_use]
pub fn issue(signing: &SigningKey, email: &str, issued: i64, expires: i64) -> String {
    issue_plan(signing, email, "", issued, expires)
}

/// Emite uma chave assinada para um plano (usado pela ferramenta do vendedor e pelos testes).
#[must_use]
pub fn issue_plan(
    signing: &SigningKey,
    email: &str,
    plan: &str,
    issued: i64,
    expires: i64,
) -> String {
    let payload = Payload {
        v: FORMAT_VERSION,
        p: PRODUCT.to_owned(),
        l: plan.to_owned(),
        e: email.to_owned(),
        i: issued,
        x: expires,
    };
    // Serializar uma struct simples não falha.
    let bytes = serde_json::to_vec(&payload).unwrap_or_default();
    let signature = signing.sign(&bytes);
    format!(
        "{PREFIX}.{}.{}",
        URL_SAFE_NO_PAD.encode(&bytes),
        URL_SAFE_NO_PAD.encode(signature.to_bytes())
    )
}

/// Confere a assinatura e o conteúdo de uma chave (não olha o vencimento).
///
/// # Errors
///
/// [`LicenseError::Format`], [`LicenseError::Signature`] ou [`LicenseError::Product`].
pub fn verify(text: &str, public: &VerifyingKey) -> Result<LicenseInfo, LicenseError> {
    let compact: String = text.chars().filter(|c| !c.is_whitespace()).collect();
    let mut parts = compact.split('.');
    let (Some(PREFIX), Some(data), Some(signature), None) =
        (parts.next(), parts.next(), parts.next(), parts.next())
    else {
        return Err(LicenseError::Format);
    };
    let bytes = URL_SAFE_NO_PAD
        .decode(data)
        .map_err(|_| LicenseError::Format)?;
    let signature: [u8; 64] = URL_SAFE_NO_PAD
        .decode(signature)
        .map_err(|_| LicenseError::Format)?
        .try_into()
        .map_err(|_| LicenseError::Format)?;
    public
        .verify(&bytes, &Signature::from_bytes(&signature))
        .map_err(|_| LicenseError::Signature)?;
    let payload: Payload = serde_json::from_slice(&bytes).map_err(|_| LicenseError::Format)?;
    if payload.v != FORMAT_VERSION || payload.p != PRODUCT {
        return Err(LicenseError::Product);
    }
    Ok(LicenseInfo {
        email: payload.e,
        plan: payload.l,
        issued: payload.i,
        expires: payload.x,
    })
}

/// Agora, em segundos desde 1970.
#[must_use]
pub fn now() -> i64 {
    Utc::now().timestamp()
}

/// Licença guardada num arquivo da pasta de dados do app.
#[derive(Debug, Clone)]
pub struct FileLicense {
    path: PathBuf,
    /// `None` se a chave embutida estiver corrompida: nenhuma chave confere.
    public: Option<VerifyingKey>,
}

impl FileLicense {
    /// Licença que usa a chave pública embutida no app.
    #[must_use]
    pub fn new(path: &Path) -> Self {
        Self {
            path: path.to_path_buf(),
            public: VerifyingKey::from_bytes(&EMBEDDED_PUBLIC_KEY).ok(),
        }
    }

    /// Licença com outra chave pública (testes).
    #[must_use]
    pub fn with_public_key(path: &Path, public: VerifyingKey) -> Self {
        Self {
            path: path.to_path_buf(),
            public: Some(public),
        }
    }

    fn saved(&self) -> Option<LicenseInfo> {
        let text = fs::read_to_string(&self.path).ok()?;
        verify(&text, self.public.as_ref()?).ok()
    }

    /// Confere a chave e, se estiver válida e dentro do prazo, guarda no computador.
    ///
    /// # Errors
    ///
    /// [`LicenseError`] se a chave for inválida ou estiver vencida; falhas ao gravar o arquivo
    /// viram [`io::Error`] no segundo item.
    pub fn activate(&self, text: &str, now: i64) -> Result<LicenseInfo, ActivateError> {
        let public = self.public.as_ref().ok_or(LicenseError::Signature)?;
        let info = verify(text, public)?;
        if now >= info.expires {
            return Err(LicenseError::Expired.into());
        }
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }
        let compact: String = text.chars().filter(|c| !c.is_whitespace()).collect();
        let temp = self.path.with_extension("tmp");
        fs::write(&temp, compact)?;
        fs::rename(&temp, &self.path)?;
        Ok(info)
    }

    /// Apaga a chave guardada (volta ao modo gratuito).
    ///
    /// # Errors
    ///
    /// Erro de E/S, exceto se o arquivo já não existir.
    pub fn remove(&self) -> io::Result<()> {
        match fs::remove_file(&self.path) {
            Err(e) if e.kind() != io::ErrorKind::NotFound => Err(e),
            _ => Ok(()),
        }
    }

    /// Situação da licença guardada num instante.
    #[must_use]
    pub fn status_at(&self, now: i64) -> LicenseStatus {
        self.saved()
            .map_or_else(LicenseStatus::free, |info| info.status_at(now))
    }
}

/// Falha ao ativar uma chave.
#[derive(Debug, thiserror::Error)]
pub enum ActivateError {
    /// A chave não vale.
    #[error(transparent)]
    License(#[from] LicenseError),
    /// Não foi possível guardar a chave no computador.
    #[error("não foi possível guardar a chave: {0}")]
    Io(#[from] io::Error),
}

impl LicenseCheck for FileLicense {
    fn is_premium(&self) -> bool {
        self.status_at(now()).premium
    }

    fn status(&self) -> LicenseStatus {
        self.status_at(now())
    }
}

/// A licença que o app usa quando ninguém escolhe outra.
#[must_use]
pub fn default_license(path: &Path) -> Box<dyn LicenseCheck> {
    if cfg!(debug_assertions) && std::env::var_os("GREAT_COTW_PREMIUM_DEV").is_some() {
        Box::new(FixedLicense(true))
    } else {
        Box::new(FileLicense::new(path))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn keys() -> (SigningKey, VerifyingKey) {
        let signing = SigningKey::from_bytes(&[7; 32]);
        let public = signing.verifying_key();
        (signing, public)
    }

    const NOW: i64 = 1_800_000_000;

    #[test]
    fn a_key_round_trips_with_its_data() {
        let (signing, public) = keys();
        let text = issue(&signing, "ana@exemplo.com", NOW, NOW + 365 * DAY);
        assert!(text.starts_with("GCO1."));
        let info = verify(&text, &public).unwrap();
        assert_eq!(info.email, "ana@exemplo.com");
        assert_eq!((info.issued, info.expires), (NOW, NOW + 365 * DAY));
        // Espaços e quebras de linha da colagem não atrapalham.
        let pasted = format!("  {}\n{}  ", &text[..30], &text[30..]);
        assert_eq!(verify(&pasted, &public).unwrap(), info);
    }

    #[test]
    fn a_tampered_or_foreign_key_is_refused() {
        let (signing, public) = keys();
        let text = issue(&signing, "ana@exemplo.com", NOW, NOW + DAY);
        let other = SigningKey::from_bytes(&[9; 32]).verifying_key();
        assert_eq!(verify(&text, &other), Err(LicenseError::Signature));
        let mut parts: Vec<&str> = text.split('.').collect();
        let forged = issue(&signing, "outra@exemplo.com", NOW, NOW + 10_000 * DAY);
        let forged_data = forged.split('.').nth(1).unwrap();
        parts[1] = forged_data;
        assert_eq!(
            verify(&parts.join("."), &public),
            Err(LicenseError::Signature)
        );
    }

    #[test]
    fn broken_text_is_a_format_error() {
        let (_, public) = keys();
        for bad in [
            "",
            "GCO1",
            "GCO1.a.b",
            "XXXX.a.b",
            "GCO1.%%%.%%%",
            "GCO1.a.b.c",
        ] {
            assert_eq!(verify(bad, &public), Err(LicenseError::Format), "{bad}");
        }
    }

    #[test]
    fn a_key_for_another_product_or_version_is_refused() {
        let (signing, public) = keys();
        let payload = |v: u8, p: &str| {
            let bytes = serde_json::to_vec(&Payload {
                v,
                p: p.into(),
                l: String::new(),
                e: "x".into(),
                i: 0,
                x: 1,
            })
            .unwrap();
            format!(
                "GCO1.{}.{}",
                URL_SAFE_NO_PAD.encode(&bytes),
                URL_SAFE_NO_PAD.encode(signing.sign(&bytes).to_bytes())
            )
        };
        assert_eq!(
            verify(&payload(1, "outro"), &public),
            Err(LicenseError::Product)
        );
        assert_eq!(
            verify(&payload(2, PRODUCT), &public),
            Err(LicenseError::Product)
        );
    }

    #[test]
    fn a_key_carries_its_plan_and_the_plan_lengths_are_a_month_and_a_year() {
        let (signing, public) = keys();
        let monthly = issue_plan(
            &signing,
            "ana@exemplo.com",
            "monthly",
            NOW,
            NOW + MONTHLY_DAYS * DAY,
        );
        let info = verify(&monthly, &public).unwrap();
        assert_eq!(info.plan, "monthly");
        assert_eq!(info.status_at(NOW).plan.as_deref(), Some("monthly"));
        assert_eq!(info.days_left(NOW), 31);
        let yearly = issue_plan(
            &signing,
            "ana@exemplo.com",
            "yearly",
            NOW,
            NOW + YEARLY_DAYS * DAY,
        );
        assert_eq!(verify(&yearly, &public).unwrap().days_left(NOW), 366);
        // Chave sem plano continua valendo e não mostra plano.
        let plain = issue(&signing, "ana@exemplo.com", NOW, NOW + DAY);
        assert_eq!(verify(&plain, &public).unwrap().status_at(NOW).plan, None);
    }

    #[test]
    fn the_status_follows_the_expiry() {
        let info = LicenseInfo {
            email: "ana@exemplo.com".into(),
            plan: "yearly".into(),
            issued: NOW,
            expires: NOW + 100 * DAY,
        };
        let active = info.status_at(NOW);
        assert_eq!(
            (active.premium, active.state, active.days_left),
            (true, LicenseState::Active, Some(100))
        );
        assert_eq!(active.plan.as_deref(), Some("yearly"));
        let soon = info.status_at(NOW + 80 * DAY);
        assert_eq!(
            (soon.premium, soon.state, soon.days_left),
            (true, LicenseState::Expiring, Some(20))
        );
        let edge = info.status_at(info.expires - 1);
        assert_eq!((edge.premium, edge.days_left), (true, Some(1)));
        let expired = info.status_at(info.expires);
        assert_eq!(
            (expired.premium, expired.state, expired.days_left),
            (false, LicenseState::Expired, Some(0))
        );
        assert_eq!(info.days_left(info.expires + 3 * DAY), -3);
    }

    #[test]
    fn activating_saves_the_key_and_the_status_reads_it_back() {
        let (signing, public) = keys();
        let dir = tempfile::tempdir().unwrap();
        let license = FileLicense::with_public_key(&dir.path().join("sub/license.key"), public);
        assert_eq!(license.status_at(NOW), LicenseStatus::free());
        let text = issue(&signing, "ana@exemplo.com", NOW, NOW + 365 * DAY);
        let info = license.activate(&text, NOW).unwrap();
        assert_eq!(info.email, "ana@exemplo.com");
        let status = license.status_at(NOW + DAY);
        assert!(status.premium);
        assert_eq!(status.email.as_deref(), Some("ana@exemplo.com"));
        // Depois do vencimento volta ao gratuito, sem apagar o arquivo.
        assert_eq!(
            license.status_at(NOW + 400 * DAY).state,
            LicenseState::Expired
        );
        license.remove().unwrap();
        license.remove().unwrap();
        assert_eq!(license.status_at(NOW), LicenseStatus::free());
    }

    #[test]
    fn activating_refuses_bad_or_expired_keys_and_saves_nothing() {
        let (signing, public) = keys();
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("license.key");
        let license = FileLicense::with_public_key(&path, public);
        let expired = issue(&signing, "ana@exemplo.com", NOW - 400 * DAY, NOW - DAY);
        assert!(matches!(
            license.activate(&expired, NOW),
            Err(ActivateError::License(LicenseError::Expired))
        ));
        assert!(matches!(
            license.activate("lixo", NOW),
            Err(ActivateError::License(LicenseError::Format))
        ));
        assert!(!path.exists());
    }

    #[test]
    fn a_corrupted_saved_key_counts_as_free() {
        let (_, public) = keys();
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("license.key");
        fs::write(&path, "GCO1.lixo.lixo").unwrap();
        assert_eq!(
            FileLicense::with_public_key(&path, public).status_at(NOW),
            LicenseStatus::free()
        );
    }

    #[test]
    fn the_app_embeds_a_valid_public_key() {
        assert!(VerifyingKey::from_bytes(&EMBEDDED_PUBLIC_KEY).is_ok());
        let dir = tempfile::tempdir().unwrap();
        assert!(!FileLicense::new(&dir.path().join("license.key")).is_premium());
    }

    #[test]
    fn small_license_types_behave() {
        assert!(!NoLicense.is_premium());
        assert_eq!(NoLicense.status(), LicenseStatus::free());
        assert!(FixedLicense(true).is_premium());
        assert!(FixedLicense(true).status().premium);
        assert!(!FixedLicense(false).status().premium);
        assert_eq!(LicenseError::Expired.code(), "licenseExpired");
        assert_eq!(LicenseError::Format.code(), "licenseFormat");
        assert_eq!(LicenseError::Signature.code(), "licenseSignature");
        assert_eq!(LicenseError::Product.code(), "licenseProduct");
    }

    #[test]
    fn the_status_is_what_the_interface_reads() {
        let json = serde_json::to_string(&LicenseStatus::free()).unwrap();
        assert_eq!(
            json,
            r#"{"premium":false,"state":"free","email":null,"expires":null,"daysLeft":null,"plan":null}"#
        );
    }

    #[test]
    fn the_default_is_free_unless_the_dev_switch_is_set() {
        if std::env::var_os("GREAT_COTW_PREMIUM_DEV").is_none() {
            let dir = tempfile::tempdir().unwrap();
            assert!(!default_license(&dir.path().join("license.key")).is_premium());
        }
    }
}
