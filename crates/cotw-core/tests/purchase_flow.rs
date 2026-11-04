//! Compra do Premium pelo app com uma loja falsa e uma chave de teste: não precisa de save nem de
//! jogo, então roda também na CI.

#![allow(clippy::unwrap_used, reason = "arquivo só de testes")]

use std::io;
use std::sync::{Arc, Mutex};

use cotw_core::diagnostics::SearchRoots;
use cotw_core::license::{FileLicense, MONTHLY_DAYS, YEARLY_DAYS, issue_plan, now};
use cotw_core::process::ProcessList;
use cotw_core::purchase::{Claim, Store, StoreError};
use cotw_core::service::{AppService, ServiceError};
use ed25519_dalek::SigningKey;

struct Closed;

impl ProcessList for Closed {
    fn running_image_names(&self) -> io::Result<Vec<String>> {
        Ok(vec!["explorer.exe".into()])
    }
}

const TOKEN: &str = "abcdefghijklmnopqrstuvwx";

/// O que a loja falsa vai responder e o que ela recebeu.
#[derive(Default)]
struct Script {
    claim: Option<Result<Claim, &'static str>>,
    checkouts: Vec<(String, String)>,
}

#[derive(Clone, Default)]
struct FakeStore(Arc<Mutex<Script>>);

impl Store for FakeStore {
    fn checkout(&self, plan: &str, token: &str) -> Result<String, StoreError> {
        self.0
            .lock()
            .unwrap()
            .checkouts
            .push((plan.into(), token.into()));
        Ok("https://mp.example/checkout/1".into())
    }

    fn claim(&self, _token: &str) -> Result<Claim, StoreError> {
        match self.0.lock().unwrap().claim.clone() {
            Some(Ok(claim)) => Ok(claim),
            Some(Err(why)) => Err(StoreError::Offline(why.into())),
            None => Ok(Claim::Pending),
        }
    }
}

struct Env {
    dir: tempfile::TempDir,
    service: AppService,
    store: FakeStore,
    signing: SigningKey,
}

fn env() -> Env {
    let dir = tempfile::tempdir().unwrap();
    let signing = SigningKey::from_bytes(&[7; 32]);
    let store = FakeStore::default();
    let license_path = dir.path().join("license.key");
    let service = AppService::new(
        dir.path(),
        SearchRoots {
            documents: Vec::new(),
            steam: Vec::new(),
        },
        Box::new(Closed),
    )
    .with_license_file(FileLicense::with_public_key(
        &license_path,
        signing.verifying_key(),
    ))
    .with_store(Box::new(store.clone()));
    Env {
        dir,
        service,
        store,
        signing,
    }
}

fn ready(env: &Env, plan: &str, days: i64) -> Claim {
    Claim::Ready {
        key: issue_plan(
            &env.signing,
            "comprador@exemplo.com",
            plan,
            now(),
            now() + days * 86_400,
        ),
        plan: plan.into(),
    }
}

#[test]
fn a_default_service_has_a_closed_store() {
    let dir = tempfile::tempdir().unwrap();
    let service = AppService::new(
        dir.path(),
        SearchRoots {
            documents: Vec::new(),
            steam: Vec::new(),
        },
        Box::new(Closed),
    );
    let error = service.start_purchase("yearly", TOKEN).unwrap_err();
    assert_eq!(error.code(), "storeClosed");
    assert_eq!(service.check_purchase().unwrap().state, "none");
}

#[test]
fn paying_activates_premium_without_pasting_a_key() {
    let env = env();
    assert!(!env.service.license_status().premium);
    assert_eq!(env.service.check_purchase().unwrap().state, "none");

    let url = env.service.start_purchase("yearly", TOKEN).unwrap();
    assert_eq!(url, "https://mp.example/checkout/1");
    assert_eq!(
        env.store.0.lock().unwrap().checkouts,
        vec![("yearly".to_owned(), TOKEN.to_owned())]
    );

    // Ainda não pagou: continua esperando, sem Premium.
    let waiting = env.service.check_purchase().unwrap();
    assert_eq!(
        (waiting.state, waiting.plan.as_deref()),
        ("waiting", Some("yearly"))
    );
    assert!(!waiting.license.premium);

    // Pagou: a chave chega e o Premium liga.
    let claim = ready(&env, "yearly", YEARLY_DAYS);
    env.store.0.lock().unwrap().claim = Some(Ok(claim));
    let done = env.service.check_purchase().unwrap();
    assert_eq!(done.state, "activated");
    assert!(done.license.premium);
    assert_eq!(done.license.plan.as_deref(), Some("yearly"));
    assert!(env.service.license_status().premium);

    // A compra terminou: nada mais a conferir.
    assert_eq!(env.service.check_purchase().unwrap().state, "none");
}

#[test]
fn a_purchase_survives_closing_the_app() {
    let env = env();
    env.service.start_purchase("monthly", TOKEN).unwrap();
    // Um serviço novo na mesma pasta (o app foi fechado e aberto) ainda espera o pagamento.
    let reopened = AppService::new(
        env.dir.path(),
        SearchRoots {
            documents: Vec::new(),
            steam: Vec::new(),
        },
        Box::new(Closed),
    )
    .with_license_file(FileLicense::with_public_key(
        &env.dir.path().join("license.key"),
        env.signing.verifying_key(),
    ))
    .with_store(Box::new(env.store.clone()));
    assert_eq!(reopened.check_purchase().unwrap().state, "waiting");
    let claim = ready(&env, "monthly", MONTHLY_DAYS);
    env.store.0.lock().unwrap().claim = Some(Ok(claim));
    assert!(reopened.check_purchase().unwrap().license.premium);
}

#[test]
fn offline_keeps_the_purchase_so_it_can_be_checked_again() {
    let env = env();
    env.service.start_purchase("yearly", TOKEN).unwrap();
    env.store.0.lock().unwrap().claim = Some(Err("sem rede"));
    let error = env.service.check_purchase().unwrap_err();
    assert!(matches!(error, ServiceError::Store(StoreError::Offline(_))));
    assert_eq!(error.code(), "storeOffline");
    env.store.0.lock().unwrap().claim = None;
    assert_eq!(env.service.check_purchase().unwrap().state, "waiting");
}

#[test]
fn a_key_that_does_not_verify_ends_the_purchase_with_an_error() {
    let env = env();
    env.service.start_purchase("yearly", TOKEN).unwrap();
    let other = SigningKey::from_bytes(&[9; 32]);
    let forged = issue_plan(&other, "x@y.z", "yearly", now(), now() + 86_400);
    env.store.0.lock().unwrap().claim = Some(Ok(Claim::Ready {
        key: forged,
        plan: "yearly".into(),
    }));
    assert_eq!(
        env.service.check_purchase().unwrap_err().code(),
        "licenseSignature"
    );
    assert!(!env.service.license_status().premium);
    assert_eq!(env.service.check_purchase().unwrap().state, "none");
}

#[test]
fn invalid_requests_never_reach_the_store_and_cancel_forgets_the_purchase() {
    let env = env();
    for (plan, token) in [("weekly", TOKEN), ("yearly", "curto"), ("", TOKEN)] {
        let error = env.service.start_purchase(plan, token).unwrap_err();
        assert_eq!(error.code(), "invalidPurchase");
    }
    assert_eq!(env.store.0.lock().unwrap().checkouts.len(), 0);

    env.service.start_purchase("yearly", TOKEN).unwrap();
    env.service.cancel_purchase();
    assert_eq!(env.service.check_purchase().unwrap().state, "none");
}
