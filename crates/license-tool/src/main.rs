//! `gco-license`: ferramenta do vendedor do Great COTW One.
//!
//! ```text
//! gco-license keygen <pasta>                       gera private.key (SECRETA) e public.key
//! gco-license issue <private.key> <e-mail> <monthly|yearly|dias>   emite uma chave do plano
//! gco-license inspect <public.key> <chave>         mostra o que há dentro de uma chave
//! ```
//!
//! A `private.key` é quem assina as licenças: nunca a coloque no repositório nem a envie a
//! ninguém. A `public.key` vai para `crates/cotw-core/src/license_public.key`.

use std::env;
use std::fs;
use std::path::Path;
use std::process::ExitCode;

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD;
use cotw_core::license::{MONTHLY_DAYS, YEARLY_DAYS, issue_plan, now, verify};
use ed25519_dalek::{SigningKey, VerifyingKey};
use rand_core::OsRng;

const DAY: i64 = 86_400;

fn read_signing(path: &str) -> Result<SigningKey, String> {
    let text = fs::read_to_string(path).map_err(|e| format!("não foi possível ler {path}: {e}"))?;
    let bytes: [u8; 32] = STANDARD
        .decode(text.trim())
        .map_err(|_| "private.key inválida".to_owned())?
        .try_into()
        .map_err(|_| "private.key com tamanho errado".to_owned())?;
    Ok(SigningKey::from_bytes(&bytes))
}

fn read_public(path: &str) -> Result<VerifyingKey, String> {
    let bytes: [u8; 32] = fs::read(path)
        .map_err(|e| format!("não foi possível ler {path}: {e}"))?
        .try_into()
        .map_err(|_| "public.key com tamanho errado".to_owned())?;
    VerifyingKey::from_bytes(&bytes).map_err(|_| "public.key inválida".to_owned())
}

fn keygen(dir: &str) -> Result<(), String> {
    let folder = Path::new(dir);
    let private = folder.join("private.key");
    if private.exists() {
        return Err(format!(
            "{} já existe: não vou sobrescrever uma chave privada",
            private.display()
        ));
    }
    fs::create_dir_all(folder).map_err(|e| e.to_string())?;
    let signing = SigningKey::generate(&mut OsRng);
    fs::write(&private, STANDARD.encode(signing.to_bytes())).map_err(|e| e.to_string())?;
    fs::write(
        folder.join("public.key"),
        signing.verifying_key().to_bytes(),
    )
    .map_err(|e| e.to_string())?;
    println!("Chaves criadas em {}", folder.display());
    println!("- private.key: SECRETA. Guarde uma cópia segura; quem a tiver emite licenças.");
    println!("- public.key: copie para crates/cotw-core/src/license_public.key e recompile o app.");
    Ok(())
}

/// Plano e dias de validade a partir do texto (`monthly`, `yearly` ou um número de dias).
fn plan_days(text: &str) -> Result<(&'static str, i64), String> {
    match text {
        "monthly" | "mensal" => Ok(("monthly", MONTHLY_DAYS)),
        "yearly" | "anual" => Ok(("yearly", YEARLY_DAYS)),
        other => other
            .parse::<i64>()
            .ok()
            .filter(|days| *days > 0)
            .map(|days| ("", days))
            .ok_or_else(|| "plano inválido: use monthly, yearly ou um número de dias".to_owned()),
    }
}

fn issue_key(args: &[String]) -> Result<(), String> {
    let [private, email, plan] = args else {
        return Err("uso: gco-license issue <private.key> <e-mail> <monthly|yearly|dias>".into());
    };
    let (name, days) = plan_days(plan)?;
    let issued = now();
    println!(
        "{}",
        issue_plan(
            &read_signing(private)?,
            email,
            name,
            issued,
            issued + days * DAY
        )
    );
    Ok(())
}

fn inspect(args: &[String]) -> Result<(), String> {
    let [public, key] = args else {
        return Err("uso: gco-license inspect <public.key> <chave>".into());
    };
    let info = verify(key, &read_public(public)?).map_err(|e| e.to_string())?;
    let left = info.days_left(now());
    println!(
        "E-mail: {}  Plano: {}",
        info.email,
        if info.plan.is_empty() {
            "-"
        } else {
            &info.plan
        }
    );
    println!(
        "Emitida: {}  Vence: {}  (faltam {left} dias)",
        info.issued, info.expires
    );
    Ok(())
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    let result = match args.split_first() {
        Some((command, rest)) if command == "keygen" => match rest {
            [dir] => keygen(dir),
            _ => Err("uso: gco-license keygen <pasta>".into()),
        },
        Some((command, rest)) if command == "issue" => issue_key(rest),
        Some((command, rest)) if command == "inspect" => inspect(rest),
        _ => Err("comandos: keygen, issue, inspect (veja o começo de main.rs)".into()),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("erro: {message}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_plans_map_to_their_lengths() {
        assert_eq!(plan_days("monthly"), Ok(("monthly", 31)));
        assert_eq!(plan_days("mensal"), Ok(("monthly", 31)));
        assert_eq!(plan_days("yearly"), Ok(("yearly", 366)));
        assert_eq!(plan_days("anual"), Ok(("yearly", 366)));
        assert_eq!(plan_days("10"), Ok(("", 10)));
        assert!(plan_days("0").is_err());
        assert!(plan_days("semanal").is_err());
    }
}
