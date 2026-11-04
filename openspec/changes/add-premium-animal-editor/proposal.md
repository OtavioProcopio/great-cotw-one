## Why

Muitos jogadores querem mais troféus de diamante, Great Ones ou pelagens raras sem passar centenas de horas procurando. O save guarda cada animal (sexo, peso, pontuação, marca de Great One, semente da pelagem), e editar esses valores no mesmo tamanho do arquivo já funcionou no jogo. Esta é a função paga do app.

## What Changes

- Editor de animais por reserva → espécie → grupo, com sexo, peso, pontuação, pelagem e Great One.
- Atalhos: deixar um animal diamante, transformar em Great One, "1 diamante por grupo", desfazer por espécie.
- Escolher a pelagem de um animal pela lista de pelagens da espécie (com chance e raridade).
- Só animais que já existem: o app não cria nem remove animais.
- Integração com o mapa: "Editar este grupo" no rastreador e "Ver no mapa" no editor.
- Plano mensal (R$ 15,00) ou anual (R$ 75,00), com uma tela que mostra tudo o que o Premium dá a mais que o gratuito.
- Tudo o que grava no jogo é premium: editar animais e também **liberar as zonas de necessidade** (por reserva, por animal ou por necessidade). Na versão gratuita o editor e a tela de liberar ficam visíveis, em modo leitura, e só gravam com a licença ativa.

## Capabilities

### New Capabilities
- `animal-editor`: editar animais que já existem (valores e pelagem), com regras de segurança.
- `premium-access`: ativação e verificação da licença premium e o que ela libera.

### Modified Capabilities
(nenhuma — `reserve-map` ganha só um atalho de interface, sem mudar requisitos)

## Impact

- Escreve `animal_population_N` pela escrita segura, sempre no mesmo tamanho (nenhum animal é criado ou removido), e `found_need_zones_adf` na liberação das zonas.
- Novo serviço de licença (fornecedor a definir) e armazenamento local da licença.
- Depende de `bootstrap-app-foundation` e `add-reserve-map-and-zone-unlock`.
