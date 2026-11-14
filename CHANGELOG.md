# ALEXANDRIA — histórico de versões

## v7 — iniciar jogo com tela de carregamento
- Ao iniciar um jogo, aparece uma tela de carregamento (capa, fundo desfocado e barra) até o jogo abrir.
- Quando o jogo abre, o ALEXANDRIA minimiza; quando o jogo fecha, ele volta e atualiza a lista (recentes e tempo de jogo).
- Steam: a abertura e o fechamento são confirmados pelo próprio registro da Steam (jogo em execução).
- Jogos abertos por executável (manuais e GOG): acompanhados pelo processo; a tela some quando a janela do jogo toma o foco.
- Epic (aberto pelo launcher da Epic): a tela de carregamento some quando o jogo toma o foco, mas não há como saber quando ele fecha; por isso não minimiza sozinho.
- Opção nos ajustes: "Minimizar quando o jogo abrir (e voltar ao fechar)".

## v6.2 — logo na abertura e README
- A abertura da tela cheia agora mostra a logo (o número 3 em algarismos romanos, III) se formando:
  as hastes crescem do centro, as serifas abrem e a logo ganha um brilho. Substitui o nome do app.
- O ícone do programa passou a ser o mesmo III.
- Adicionado README.md (em inglês) com descrição e instruções de build.

## v6.1 — abertura da tela cheia e correção de bordas
- Animação ao entrar no modo tela cheia (escuro, brilho na cor de destaque, linha de luz e o nome do app),
  com som de abertura sintetizado (acorde etéreo com cintilações). Opção "Som de abertura da tela cheia" nos ajustes.
- A animação respeita o ajuste de animações e a redução de movimento do Windows.
- Corrigido: a primeira e a última capa da fileira apareciam cortadas nas bordas da tela cheia.

## v6 — lojas, tempo de jogo, teclado na tela e mais
- Abrir já em tela cheia agora é uma opção nos ajustes (desligada por padrão); também há "Iniciar com o Windows".
- Epic Games e GOG: jogos instalados entram na biblioteca como os da Steam (sincronização por loja).
- Tempo de jogo: lido da própria Steam e contado quando o jogo abre direto por executável.
- Teclado na tela: com o controle, confirmar em um campo de texto abre um teclado navegável.
- IGDB também traz capas e artes (além das descrições); o buscador de imagens ganhou abas SteamGridDB e IGDB.
- Sons de navegação discretos (opcional, desligados por padrão).
- Backup e restauração da biblioteca (sem chaves de API); importar nunca sobrescreve.
- Medição de desempenho sob demanda e verificação de atualização (exige repositório público com versão publicada).
- O instalador agora também publica a versão no GitHub (Releases). A abertura passa sempre pelo desktop, a menos que a opção esteja ligada.

## v5.1 — versão visível
- O número da versão agora aparece nos ajustes e no nome do instalador (ALEXANDRIA_0.5.1_x64-setup.exe),
  para saber exatamente qual versão está instalada.

## v5 — correção da tela preta
- Corrigido: o fundo dinâmico podia ficar sem camadas quando a imagem trocava duas vezes em menos de meio segundo
  (acontecia quando a Steam terminava de baixar capas), derrubando a interface e deixando a tela preta.
- Nova proteção: se a interface quebrar, aparece uma mensagem com o erro e o botão Recarregar, em vez de tela preta.

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
