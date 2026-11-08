# Como publicar uma versão

O app instalado procura atualizações em
`https://github.com/OtavioProcopio/great-cotw-one-site/releases/latest/download/latest.json`
e só instala o que estiver assinado com a **chave de atualização** (a pública está em
`src-tauri/tauri.conf.json`, a privada fica fora do repositório).

## Uma vez só
1. **Chave de atualização:** está em `C:\Users\<você>\.great-cotw-one\updater-keys\`
   (`updater.key`, `password.txt`, `updater.key.pub`). Guarde uma cópia num cofre de senhas.
   **Perder a privada** impede atualizar quem já instalou (teriam de baixar o instalador de novo).
   **Vazar a privada** deixa outra pessoa empurrar atualizações: troque o par e publique uma versão nova.
2. **Segredos do repositório do app** (Settings → Secrets and variables → Actions):
   `TAURI_SIGNING_PRIVATE_KEY`, `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`, `SITE_RELEASES_TOKEN`
   (token fino com *Contents: Read and write* só em `great-cotw-one-site`).
3. **Variável** `GREAT_COTW_LICENSE_SERVER` com o endereço `https://` do servidor de licenças
   (sem ela o botão de compra avisa que a loja não abriu).

## A cada versão
1. Suba a versão nos dois lugares, igual: `Cargo.toml` (`[workspace.package] version`) e
   `src-tauri/tauri.conf.json` (`version`). Siga [SemVer](https://semver.org/lang/pt-BR/):
   correção → 0.1.**1**; novidade → 0.**2**.0.
2. Escreva o que mudou em `release-notes.md` (aparece no aviso de atualização do app e na página de download).
3. Rode tudo local: `cargo lint`, `cargo test --workspace`, `cd ui; npm run lint; npm test`.
4. Commit, `git tag v0.2.0`, `git push origin main v0.2.0`.
5. O workflow **Release** confere se a tag bate com as versões, compila, assina, gera `latest.json`
   e publica tudo nas Releases do repositório do site. Em minutos, quem tem o app vê o aviso.

## Compilar local sem assinar
`npm run tauri build` (sem `--config src-tauri/tauri.release.conf.json`) gera o instalador
normal, sem os arquivos de atualização. Para gerar os de atualização local:
```powershell
$env:TAURI_SIGNING_PRIVATE_KEY = Get-Content $HOME\.great-cotw-one\updater-keys\updater.key -Raw
$env:TAURI_SIGNING_PRIVATE_KEY_PASSWORD = Get-Content $HOME\.great-cotw-one\updater-keys\password.txt -Raw
ui\node_modules\.bin\tauri build --config src-tauri\tauri.release.conf.json
```

## Se algo der errado
- **Versão com defeito publicada:** publique logo outra com o número maior (o app não volta versões).
  Enquanto isso, apague o `latest.json` da release ruim para ninguém mais receber.
- **Workflow falhou na assinatura:** confira os dois segredos `TAURI_SIGNING_*`.
