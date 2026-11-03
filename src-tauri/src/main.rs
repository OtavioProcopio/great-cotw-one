//! Ponto de entrada do Great COTW One; toda a lógica fica em `great_cotw_one_lib`.

// Sem janela de console na versão final no Windows.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    great_cotw_one_lib::run();
}
