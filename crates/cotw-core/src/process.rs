//! Detecção do processo do jogo (spec `safe-save-writing`: jogo fechado antes de gravar).

use std::io;
use std::process::Command;

/// Nome do executável do jogo.
pub const GAME_PROCESS: &str = "theHunterCotW_F.exe";

/// Fonte da lista de processos em execução (substituível nos testes).
pub trait ProcessList {
    /// Nomes das imagens dos processos em execução.
    ///
    /// # Errors
    ///
    /// Retorna o erro de E/S se a lista não puder ser obtida.
    fn running_image_names(&self) -> io::Result<Vec<String>>;
}

/// Lista real do Windows, via `tasklist`.
#[derive(Debug, Clone, Copy, Default)]
pub struct SystemProcessList;

impl ProcessList for SystemProcessList {
    fn running_image_names(&self) -> io::Result<Vec<String>> {
        let mut command = Command::new("tasklist");
        command.args(["/FO", "CSV", "/NH"]);
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            command.creation_flags(0x0800_0000); // CREATE_NO_WINDOW
        }
        let output = command.output()?;
        if !output.status.success() {
            return Err(io::Error::other(format!(
                "tasklist terminou com {}",
                output.status
            )));
        }
        Ok(parse_tasklist_csv(&String::from_utf8_lossy(&output.stdout)))
    }
}

/// Primeiro campo (nome da imagem) de cada linha da saída CSV do `tasklist`.
#[must_use]
pub fn parse_tasklist_csv(output: &str) -> Vec<String> {
    output
        .lines()
        .filter_map(|line| {
            let rest = line.trim().strip_prefix('"')?;
            Some(rest.split('"').next()?.to_owned())
        })
        .collect()
}

/// Por que a gravação não pode continuar.
#[derive(Debug, thiserror::Error)]
pub enum GameStateError {
    /// O jogo está aberto.
    #[error("o jogo está aberto ({GAME_PROCESS}); feche-o antes de gravar")]
    Running,
    /// Não foi possível saber se o jogo está aberto; por segurança, a gravação é bloqueada.
    #[error("não foi possível verificar se o jogo está aberto: {0}")]
    Unknown(#[from] io::Error),
}

/// Confere que o jogo está fechado.
///
/// # Errors
///
/// [`GameStateError::Running`] se o processo existir; [`GameStateError::Unknown`] se a lista de
/// processos não puder ser obtida (na dúvida, não grava).
pub fn ensure_game_closed(processes: &dyn ProcessList) -> Result<(), GameStateError> {
    let running = processes
        .running_image_names()?
        .iter()
        .any(|name| name.eq_ignore_ascii_case(GAME_PROCESS));
    if running {
        Err(GameStateError::Running)
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Fake(io::Result<Vec<&'static str>>);

    impl ProcessList for Fake {
        fn running_image_names(&self) -> io::Result<Vec<String>> {
            match &self.0 {
                Ok(names) => Ok(names.iter().map(|n| (*n).to_owned()).collect()),
                Err(e) => Err(io::Error::new(e.kind(), e.to_string())),
            }
        }
    }

    #[test]
    fn parses_tasklist_csv_lines() {
        let out = "\"System Idle Process\",\"0\",\"Services\",\"0\",\"8 K\"\r\n\"theHunterCotW_F.exe\",\"4242\",\"Console\",\"1\",\"3.000 K\"\r\n\r\n";
        assert_eq!(
            parse_tasklist_csv(out),
            ["System Idle Process", "theHunterCotW_F.exe"]
        );
    }

    #[test]
    fn ignores_messages_that_are_not_csv() {
        let out = "INFORMAÇÕES: nenhuma tarefa está em execução.\n";
        assert_eq!(parse_tasklist_csv(out), Vec::<String>::new());
    }

    #[test]
    fn running_game_blocks_writing_case_insensitively() {
        let list = Fake(Ok(vec!["explorer.exe", "THEHUNTERCOTW_F.EXE"]));
        assert!(matches!(
            ensure_game_closed(&list),
            Err(GameStateError::Running)
        ));
    }

    #[test]
    fn other_processes_do_not_block() {
        let list = Fake(Ok(vec!["explorer.exe", "theHunterCotW.exe"]));
        assert!(ensure_game_closed(&list).is_ok());
    }

    #[test]
    fn unknown_state_blocks_writing_too() {
        let list = Fake(Err(io::Error::other("sem tasklist")));
        assert!(matches!(
            ensure_game_closed(&list),
            Err(GameStateError::Unknown(_))
        ));
    }

    #[test]
    fn real_process_list_works_on_this_machine() {
        // Em qualquer máquina Windows há processos em execução; o jogo geralmente não está.
        let names = SystemProcessList.running_image_names().unwrap();
        assert!(names.iter().any(|n| n.eq_ignore_ascii_case("explorer.exe")));
    }
}
