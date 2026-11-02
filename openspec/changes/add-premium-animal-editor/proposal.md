## Why

Muitos jogadores querem mais troféus de diamante, Great Ones ou pelagens raras sem passar centenas de horas procurando. O save guarda cada animal (sexo, peso, pontuação, marca de Great One, semente da pelagem), e editar esses valores no mesmo tamanho do arquivo já funcionou no jogo. Esta é a função paga do app.

## What Changes

- Editor de animais por reserva → espécie → grupo, com sexo, peso, pontuação, pelagem e Great One.
- Atalhos: deixar um animal diamante, transformar em Great One, "1 diamante por grupo", desfazer por espécie.
- Criar animal num grupo (experimental), com limite de tamanho por espécie e reconstrução completa do arquivo.
- Integração com o mapa: "Editar este grupo" no rastreador e "Ver no mapa" no editor.
- Liberação premium: o editor fica visível na versão gratuita em modo leitura e só grava com a licença ativa.

## Capabilities

### New Capabilities
- `animal-editor`: editar e criar animais do save com regras de segurança.
- `premium-access`: ativação e verificação da licença premium e o que ela libera.

### Modified Capabilities
(nenhuma — `reserve-map` ganha só um atalho de interface, sem mudar requisitos)

## Impact

- Escreve `animal_population_N` pela escrita segura; criar animal muda o tamanho do arquivo e exige o escritor ADF com cadeia de relocação.
- Novo serviço de licença (fornecedor a definir) e armazenamento local da licença.
- Depende de `bootstrap-app-foundation` e `add-reserve-map-and-zone-unlock`.
