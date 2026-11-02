# Implementações de referência

Código usado nos protótipos (scripts e páginas) e validado num save real nível 60. Não é o código do app: serve de oráculo para os testes do núcleo em Rust.

| Arquivo | O que faz |
|---|---|
| `hash.py` | Jenkins `hashlittle` (lookup3). Nomes de arquivo nos `.tab`, hashes de equipamento, habilidades, conquistas. |
| `adf.py` | Leitor genérico de ADF → JSON (`python adf.py arquivo.bin [-t]`). Espera o ADF já descomprimido. |
| `aaf.py`, `sarc.py` | Bundles AAF (deflate em blocos) e SARC (lista de arquivos). |
| `rtpc.py` | RTPC (`.blo`, `settings/player.bin`). |
| `pop.py` | `animal_population_N`: animais, grupos; `add_animals` **não** reconstrói a cadeia (ver aviso no arquivo). |
| `nzio.py`, `nzwrite.py`, `nzfile.py` | Ler e reconstruir `found_need_zones_adf` (com cadeia de relocação). |
| `wiwrite.py` | Ler e reconstruir `worlditemsdata_adf`. |
| `core.js` | Navegador: ler/gravar `animal_population_N` incluindo criação de animais **com** cadeia corrigida, deflate/inflate e zip. |

Para descomprimir um save: pular 32 bytes, `zlib.decompress`, pular 5 bytes → ADF.
