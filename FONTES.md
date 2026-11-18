# Fontes da Loja (catálogos)

A aba **Loja** mostra os jogos de catálogos que **você** cadastra. O ALEXANDRIA não traz fontes
prontas e não indica fontes. Use só catálogos de conteúdo que você tem o direito de baixar e usar:
seus próprios jogos, projetos de código aberto, jogos gratuitos cedidos pelos autores, demos.

## Como funciona
1. Em Loja > Fontes, cole o endereço (https) de um arquivo de catálogo e confirme.
2. Os jogos do catálogo aparecem na Loja. Clique em **Baixar**.
3. Você vê a porcentagem, a velocidade e o tempo que falta. Dá para **pausar** e continuar depois.
4. Se o catálogo informar o **SHA-256** do jogo, o arquivo só é instalado se bater; se não bater, é descartado.
   Sem SHA-256, o jogo é instalado **sem verificação** (a Loja avisa).
5. O jogo entra na biblioteca. Em **Remover**, a pasta dele é apagada.

## Formato do catálogo (JSON)
```json
{
  "format": 1,
  "name": "Meus jogos",
  "games": [
    {
      "id": "meu-jogo",
      "name": "Meu Jogo",
      "version": "1.0.0",
      "description": "Um texto curto.",
      "category": "Plataforma",
      "license": "MIT",
      "size": 123456789,
      "cover": "https://exemplo.com/capa.jpg",
      "url": "https://exemplo.com/meu-jogo-1.0.0.zip",
      "sha256": "COLE_AQUI_OS_64_CARACTERES_DO_SHA256",
      "type": "zip",
      "launch": "MeuJogo/jogo.exe"
    }
  ]
}
```

| Campo | Obrigatório | Observação |
|---|---|---|
| `format` | sim | sempre `1` |
| `id` | sim | minúsculas, números, ponto, hífen ou sublinhado |
| `name` | sim | nome exibido |
| `version` | não | se mudar, a Loja oferece **Atualizar** |
| `url` | sim | endereço https do arquivo |
| `sha256` | não (recomendado) | 64 caracteres hexadecimais; quando informado, o arquivo é conferido |
| `type` | não | `zip` (padrão) ou `exe` |
| `launch` | `zip`: sim | caminho relativo do .exe/.bat dentro do .zip, sem `..` |
| `size`, `description`, `category`, `license`, `cover` | não | só informativos (`cover` precisa ser https) |

## Como calcular o SHA-256 (opcional, recomendado)
No PowerShell: `Get-FileHash .\meu-jogo-1.0.0.zip -Algorithm SHA256`

## Segurança
- Só endereços **https** (redirecionamentos para http são recusados).
- O .zip é extraído com proteção contra caminhos que saem da pasta.
- A instalação é montada numa pasta temporária e só depois trocada pela final.
- Antes de baixar, o programa confere se há espaço livre no disco.
- Sem SHA-256 não há como saber se o arquivo foi alterado no caminho: prefira catálogos que o informam.
- O SHA-256 garante que o arquivo é o que o catálogo diz, **não** que ele é seguro: o jogo é um programa
  que roda no seu PC. Só adicione fontes em que você confia.
