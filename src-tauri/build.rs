//! Script de build do Tauri: gera o contexto (ícones, permissões) usado por `generate_context!`.

fn main() {
    tauri_build::build();
}
