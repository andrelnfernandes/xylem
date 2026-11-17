# Copiar arquivos de outra máquina por SSH

Este procedimento exibe `deploy@webserver` ao lado desta máquina e copia três
arquivos de backup dela para um diretório local.

## Passo a passo

1. Pressione F8. A tela se divide em duas janelas, ambas no diretório atual. A
   janela da esquerda recebe o foco.
2. Pressione `L`, digite `deploy@webserver:` e pressione Enter. A janela da
   esquerda passa a exibir o diretório pessoal daquele usuário naquela máquina.
3. Posicione o cursor sobre `backups`, pressione → para ler seu conteúdo e
   Enter para ir aos seus arquivos.
4. Pressione `T` sobre cada arquivo que deseja copiar.
5. Pressione Ctrl+C. O prompt sugere como destino o diretório da outra janela.
6. Pressione Enter.

![Os seis passos: F8, Log para deploy@webserver, os arquivos de backups, três arquivos marcados, Ctrl+C sugerindo a janela local e três arquivos copiados](../../screenshots/howto-ssh-copy.gif)

O rodapé exibe `Copied 3 of 3 files`.

## O endereço

| Endereço | Abre |
|---|---|
| `user@host:` | o diretório pessoal |
| `user@host:/srv/www` | esse diretório específico |
| `ssh://user@host:2222/srv/www` | o mesmo, em outra porta |
| `docker://container/caminho` | um container em execução, via `docker exec` |

O Xylem utiliza o comando `ssh` instalado no sistema operacional. Atalhos de
hosts, chaves, agente SSH e entradas de `known_hosts` de `~/.ssh/config` são
respeitados, e nada precisa ser instalado na máquina remota.

## Senhas

Se o ssh solicitar senha ou senha de chave (*passphrase*), a solicitação é
exibida no rodapé e a digitação permanece oculta. O Xylem repassa o valor ao
ssh sem armazená-lo. Ao conectar-se a um host pela primeira vez, o rodapé exibe
o fingerprint da chave e aguarda a confirmação com `yes`.

## Na outra direção

Para copiar arquivos em direção à máquina remota, mude para a janela local com
Tab, marque os arquivos e pressione Ctrl+C: o prompt sugerirá o diretório da
janela remota. Os comandos Move, Rename, Delete, Makedir, Attributes e New date
também funcionam no lado remoto.

## Quando não se comporta como esperado

- **A janela de arquivos exibe `Not logged`.** O diretório não foi lido. Volte
  com Esc e pressione → sobre ele. O passo 3 realiza essa leitura.
- **Ctrl+C solicita destino sem sugestão preenchida.** A tela não está dividida.
  Digite o caminho local manualmente ou pressione F8 antes de copiar.
- **Edit e eXecute não aparecem no menu.** Eles não funcionam em máquinas
  remotas. Alt+X abre um shell diretamente naquele servidor.
- **Não há lixeira.** A exclusão em máquinas remotas é definitiva, e o prompt
  alerta sobre isso.

Para retornar ao local posteriormente sem digitar o endereço, salve o diretório
como favorito pressionando Ctrl+F11.

Veja também: [máquinas remotas (SSH)](remote.md), [tela dividida](split.md),
[favoritos](bookmarks.md).
