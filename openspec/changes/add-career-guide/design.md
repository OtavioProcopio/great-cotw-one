## Context

Ver proposal.md. Mapeamentos já provados: seções 3.5, 3.6, 4.2 e 4.3 da skill de conhecimento (conquista ↔ missão por `hashlittle("ach_NNN")`, objetivos com `CountRequired`, `ValueOccurrences` para itens contados, XLS de habilidades e árvores).

## Goals / Non-Goals

**Goals:** tudo calculado localmente a partir do save e do jogo; atualização ao vivo quando o save muda (o jogo grava ao sair ou salvar).
**Non-Goals:** ler a API web da Steam (precisa de login), traduzir textos que o jogo não traz em PT.

## Decisions

1. **Nomes**: textos de missões vêm de `missions.stringlookup` no idioma do sistema (índice 7 = português). Nomes de habilidades/vantagens: chaves do jogo + tabela EN oficial mantida no app até achar a localização de UI do jogo.
2. **Progresso de conquista**: tipo "missões" (via cadeia), "contagem" (objetivo único com meta >1), "objetivos" (vários objetivos), "único". Itens que faltam: ler as condições do objetivo na definição (strings de espécie/região) e as tabelas `ValueOccurrences` do save.
3. **Status da Steam**: ler o cache binário KeyValues local da Steam; nunca pedir credenciais.
4. **Build**: escrita no mesmo tamanho (arrays inline de 128 posições); validação idêntica à do protótipo `bd.js`.
5. **Observador de arquivos** na pasta do save para recarregar as telas quando o jogo gravar.

## Risks / Trade-offs

- [Nomes de sub-regiões só como código (`sib_d_02`)] → mostrar código + posição no mapa até mapear os nomes.
- [Ordem das árvores em `SkillPointsSpent`] → validada com um save real (Perseguidor=0, Emboscador=1; Rifles, Pistolas, Espingardas, Arcos = 0..3); conferir em save com distribuição desigual antes de liberar a gravação.

## Open Questions

- Localização de UI em PT-BR dentro dos arquivos do jogo (melhora nomes, não muda specs).
