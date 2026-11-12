# Spec: favoritos com nome, e de máquinas remotas

Origem: conversa de 2026-11-12, depois do prompt vazio no `L`, no Copy e no Move (PR #106).
O autor quer dar nome a endereços difíceis de lembrar ("Servidor do Oracle", "Docker do
Postgres Amb 1") e chegar a eles a partir do `L`.

**Estado: aprovado pelo autor em 2026-11-12, com as teclas sugeridas (F2 para o nome,
F11 nos prompts), para entrar na 0.7.0. Implementado na branch `named-bookmarks`, as três
fatias juntas;** as diferenças em relação ao texto abaixo estão em "O que a implementação
mudou", no fim.

## O que existe hoje

- **Favoritos** (`src/state.rs`, `Bookmark { path, label }`): uma tecla cada (`0`–`9`,
  `a`–`z`, 36 no total), salvos em `history.toml` junto com o histórico dos prompts.
  Ctrl+F11 salva o diretório atual, `'` pula, F11 abre o gerenciador.
- **O nome já existe no arquivo, mas não na interface:** o campo `label` é lido, gravado e
  mostrado entre parênteses no gerenciador, só que nenhuma tecla o preenche. Hoje só entra
  editando o `history.toml` à mão.
- **Um favorito é sempre uma pasta local:** pular usa `jump_to_path`, que resolve o caminho
  no disco. Salvar de dentro de uma janela remota grava o caminho interno da janela, que
  não reconecta.
- **O gerenciador confere cada pasta a cada desenho** (`bm.path.is_dir()` em
  `draw_bookmarks`): num compartilhamento de rede lento, isso segura a tela.
- **O `L`** aceita endereços (`user@host:/x`, `ssh://`, `docker://`) e guarda o que foi
  digitado no histórico (↑).

## O que muda

### 1. Dar nome a um favorito

- No gerenciador (F11), uma tecla abre um prompt `NAME:` para o favorito sob o cursor,
  com o nome atual para editar. Nome vazio remove o nome.
- A lista passa a mostrar o nome em destaque, antes do caminho:
  `[o] Servidor do Oracle   oracle@db01:/u01/app`. Sem nome, fica como hoje.
- Ao salvar com Ctrl+F11, nada muda: o nome é dado depois, no gerenciador. Salvar de novo
  na mesma tecla mantém o nome se o lugar for o mesmo, e o descarta se for outro.

### 2. Favorito de máquina remota ou container

- Numa janela remota, Ctrl+F11 guarda o **endereço** da janela mais a pasta sob o cursor
  (`ssh://oracle@db01/u01/app`, `docker://postgres-amb1/var/lib/postgresql`), na forma que
  `Address::parse` lê de volta.
- Pular para ele faz o mesmo que digitar o endereço no `L`: conecta (com a pergunta de
  senha ou de `known_hosts`, se houver) e abre a janela. Se já existe uma janela nessa
  máquina e a pasta está na árvore dela, só move o cursor.
- No gerenciador, um favorito remoto mostra o endereço e não é conferido (`[not found]`
  não se aplica: conferir exigiria conectar).

### 3. A lista de favoritos a partir dos prompts

- No prompt do `L`, **F11** abre a lista de favoritos para escolher: Enter (ou a tecla do
  favorito) fecha a lista e deixa o caminho ou o endereço no prompt, sem confirmar; Esc
  volta ao prompt como estava. É o mesmo desenho do F2, que escolhe uma pasta na árvore.
- O mesmo vale no Copy e no Move. Um favorito remoto como destino segue a regra atual:
  só funciona com uma janela aberta naquela máquina, e a mensagem diz isso.
- F2 e ↑ continuam como estão.

### De passagem

- O gerenciador deixa de conferir, a cada desenho, uma pasta em montagem lenta
  (`Mount::reads_slowly`, do PR #105): ali o favorito aparece sem o `[not found]`.

## Modelo de ameaça

O `history.toml` é do usuário, mas é um arquivo: pode ter sido editado à mão, copiado de
outra máquina ou alterado por outro programa. O que está nele é lido como texto de fora.

1. **O nome é só texto na tela.** Limite de 60 caracteres; caracteres de controle e de
   direção (bidi) são recusados ao digitar e, vindos do arquivo, passam pelos filtros que
   já valem para todo texto de fora (`ui::draw`). O nome nunca vai para o `ssh`, para o
   `docker`, para um shell ou para um caminho.
2. **O endereço é validado ao usar, não ao ler.** Pular passa por `Address::parse`, a mesma
   validação do que é digitado no `L` (nome de host e de container sem `-` inicial nem
   espaço, argumentos depois de `--`). Um endereço inválido no arquivo vira mensagem de
   erro, nunca um comando.
3. **Nada conecta sozinho.** Abrir o gerenciador, desenhar a lista ou iniciar o Xylem não
   abre conexão nenhuma; só Enter ou a tecla do favorito.
4. **Nenhuma senha é guardada.** O endereço não leva senha (`Address` não tem esse campo),
   e a conexão segue pelo `ssh` do sistema, com agent, chaves e askpass como hoje.
5. **Um nome não engana sobre o destino:** o endereço real aparece sempre ao lado do nome,
   inteiro ou cortado no fim com `…`, nunca escondido.

## Formato do arquivo

```toml
[bookmarks]
p = "/home/andre/projetos"                         # como hoje
o = { path = "", address = "ssh://oracle@db01/u01/app", label = "Servidor do Oracle" }
```

- Favorito local: sem mudança (`path`, com `label` opcional).
- Favorito remoto: `address`, e `path = ""`. O `path` vazio fica de propósito: uma versão
  anterior do Xylem exige `path` em cada favorito, e sem ele deixaria de ler o arquivo
  inteiro (e gravaria por cima ao salvar o histórico). Com ele, a versão antiga mostra um
  favorito que não abre, e nada se perde.
- Um favorito com `address` e `path` não vazio é lido como remoto.

## Fatias

Cada uma funciona sozinha no `main`:

1. **Nome:** a tecla no gerenciador, o prompt, a lista com o nome em destaque, e o
   gerenciador sem conferir pastas em montagem lenta.
2. **Remoto:** `address` no arquivo, Ctrl+F11 numa janela remota, pular conectando.
3. **F11 nos prompts** do `L`, do Copy e do Move.

Ao fim de cada uma: `docs/manual/bookmarks.md` (e `treespec.md`, `copy-move.md`,
`remote.md` onde couber), nas 6 línguas.

## Testes

- Nome: dar, trocar e remover; limite de tamanho; nome com caractere de controle ou bidi
  vindo do arquivo aparece filtrado; salvar de novo na mesma tecla mantém ou descarta o
  nome conforme o lugar.
- Arquivo: ida e volta dos três formatos; arquivo antigo (só caminho) continua lido;
  favorito remoto com `path = ""`; endereço inválido vira erro ao pular, sem conectar.
- Remoto: com o `sftp-server` dos testes (`remote::for_tests`), salvar numa janela remota
  e pular de volta; pular com a janela já aberta só move o cursor.
- Prompts: F11 no `L` preenche e não confirma; Esc devolve o texto digitado; no Copy, um
  favorito remoto sem janela aberta dá a mensagem atual.
- Desenho com `TestBackend`: nome e endereço na lista, corte com `…`, favorito remoto sem
  `[not found]`.

## Fora desta proposta

- **Mais de 36 favoritos** (favoritos sem tecla): fica para quando fizer falta.
- **Descobrir máquinas e containers sozinho** (`docker ps`, `~/.ssh/config`): é o plano
  `machines.md`, na branch `plan-machines`, ainda não aprovado. Os nomes aqui não
  dependem dele.
- **Compartilhamentos de rede montados** (SMB) já são pastas locais: a fatia 1 basta.

## Decisões

1. **A tecla do nome no gerenciador é F2.** As letras e os dígitos são teclas de favorito
   ali, então precisava ser outra; na lista não há pasta para escolher, e F2 é "renomear"
   em muitos programas. A alternativa era `=`.
2. **F11 dentro dos prompts** abre a lista (fatia 3).

3. **Entra na 0.7.0** (2026-11-12), antes do release e da fase 4 do SSH.

## O que a implementação mudou

1. **O endereço fica no `path`, não num campo `address`.** O formato proposto acima perdia
   o endereço numa versão anterior do Xylem: ela lê o favorito sem conhecer o campo e, ao
   salvar o histórico (o que acontece a cada prompt confirmado), grava o arquivo de novo sem
   ele. Com o endereço no próprio `path`, a versão antiga lê e grava de volta um caminho que
   não abre, e nada se perde:

   ```toml
   [bookmarks]
   p = "/home/andre/projetos"
   o = { path = "ssh://oracle@db01/u01/app", label = "Servidor do Oracle" }
   ```

   Um favorito é remoto quando o `path` começa com `ssh://` ou `docker://`. Um caminho local
   de favorito é sempre absoluto (`/x`, `C:\x`), então nunca começa assim. O endereço é
   gravado sempre na forma `ssh://` (que aceita porta) ou `docker://`, e só se
   `Address::parse` o lê de volta igual.
2. **Pasta com espaço no caminho não vira favorito remoto:** `Address::parse` recusa espaço,
   como no `L`. O Xylem avisa e não salva.
3. **"Uma janela nessa máquina" é a janela ativa.** Se ela está na mesma máquina (mesmo
   tipo de acesso, usuário, host e porta) e a pasta está na árvore dela, só o cursor se
   move; senão, conecta nela, como o `L`.
4. **A conferência das pastas** é feita uma vez, ao abrir a lista, e não a cada desenho;
   remotas e as de montagem lenta não são conferidas.
5. **A lista aberta por F11 num prompt só escolhe:** Del, F2 e Ins não fazem nada ali.
6. **O nome vindo do arquivo** é mostrado com no máximo 60 caracteres e com controle e
   bidi trocados por `·`, além do filtro final de `ui::draw`.
