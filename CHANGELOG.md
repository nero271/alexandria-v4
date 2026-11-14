# ALEXANDRIA — histórico de versões

## v4 — ícones no lugar de textos
- Ajustes e tela cheia agora são botões só com ícone (com dica ao passar o mouse).
- Dicas do controle no fullscreen mostram o botão + um ícone, sem frases.
- Detalhes do jogo: Jogar com ícone e texto; Favoritar, Editar e Fechar só com ícone.
- Título dos ajustes virou ícone. Adicionado este histórico.

## v3 — novo visual e IGDB
- Visual refeito: barra translúcida, abas com indicador deslizante, capas com animação de mola,
  folhas de detalhes com desfoque, fullscreen com relógio, botão Jogar e fileira de capas.
- No fullscreen, confirmar já inicia o jogo. Gatilhos LT/RT (L2/R2) pulam 6 jogos.
- IGDB: busca de descrições (Client ID e Secret nos ajustes) com escolha do usuário.
- Adicionar jogo manualmente também pelos ajustes (útil no fullscreen).

## v2 — etapas 2 a 8
- Modo desktop e modo fullscreen próprios; navegação completa por controle (Xbox, DualShock 4, DualSense)
  com símbolos que trocam sozinhos.
- Steam: detecção, importação, AppIDs, jogos desinstalados, atualização ao voltar para o app, iniciar jogos.
- Capas, fundos, logos e descrições da Steam baixados sem chave; busca de imagens no SteamGridDB.
- Fundo dinâmico com troca suave e pré-carregamento; cache de imagens em disco.
- Personalização: nome, cor de destaque, transparência do véu, animações, fundo próprio.
- Abas Todos, Favoritos e Recentes; filtro por categoria; edição individual de cada jogo.

## v1 — fundamentos
- Projeto Tauri 2 + Rust + React/TypeScript + SQLite, banco com migrações versionadas.
- Biblioteca básica: adicionar, buscar, favoritar e remover, com tudo salvo.

## Ainda pendente
- Nada foi compilado nem testado em um Windows real: o primeiro build na nuvem pode apontar erros.
- Testes automáticos escritos (banco, Steam, configurações), ainda não executados.
- O ícone do programa só muda na instalação.
