# Plano: compartilhamentos de rede sem tela parada

Origem: no WSL2, um compartilhamento SMB montado com
`mount -t drvfs '\\10.51.200.61\html$' /mnt/html` deixou a tela parada por muito
tempo depois de um Log em `/mnt/html`. O Xylem não travou: estava lendo.

## A causa

A leitura em segundo plano de [cloud-opening.md](cloud-opening.md) só vale para
`fuse.rclone`. O compartilhamento aparece em `/proc/mounts` como

```
\134\13410.51.200.61\134html$ /mnt/html 9p rw,...,aname=drvfs;path=UNC\10.51.200.61\html$;...
```

e é tratado como disco local. O Log cria a árvore com `Tree::new`. `expand` lê
a raiz e cada subpasta dela na thread da tela, com um `stat` por entrada, e cada
`stat` atravessa 9p, o Windows e o SMB até o servidor.

## Decisões

1. **Quem lê devagar** (`Mount::reads_slowly`):
   - `fuse.rclone` (como já era);
   - `cifs`, `smb3` e `smbfs` (SMB pelo kernel), `fuse.sshfs` e `davfs`;
   - `9p` ou `drvfs` (WSL2 e WSL1) **só quando a origem é UNC** (`\\servidor\...`).
     `/mnt/c` também é 9p, mas é o disco da máquina; tratá-lo como lento
     desligaria o status do Git e o inotify em repositórios no Windows.
2. **NFS, AFS e Ceph ficam de fora.** São redes de LAN, com cache de atributos, e
   comuns como `$HOME`. Ali, a leitura lenta desligaria o Git automático da pasta
   de trabalho inteira. Volta à mesa se aparecer um caso real.
3. **Unidade de rede mapeada por letra** (`mount -t drvfs Z: /mnt/z`, onde `Z:` é um
   compartilhamento no Windows): do Linux, não dá para distinguir de um disco
   local sem perguntar ao Windows. Fica de fora; o caminho UNC é o recomendado.
4. **`/proc/mounts` decodificado por inteiro:** os escapes octais (`\040` espaço,
   `\011` tab, `\012` nova linha, `\134` barra invertida), não só o espaço. A origem
   UNC chega como `\134\134servidor`; um ponto de montagem com `\` ou tab no nome
   também passa a bater.
5. **O que muda nesses compartilhamentos** é o mesmo que já vale para o rclone:
   árvore lida em segundo plano (4 threads, a pasta sob o cursor primeiro), sem
   `git status` automático (Ctrl+G continua), sem inotify (F3 atualiza), e o Usage
   não separa outro disco montado dentro do compartilhamento.

## Treespec e ir a um caminho

- **`show_node`** pede a leitura da pasta de destino (`touch`) em vez de fazê-la
  (`scan`). No disco local, `touch` lê na hora, como antes. Vale para Treespec,
  Log dentro da árvore, marcadores e o `cd` do terminal.
- **Enquanto se digita no Treespec,** o caminho que passa por uma pasta lenta ainda
  não lida pede essa leitura e o cursor para nela. Quando a leitura chega, o cursor
  segue o que já foi digitado, sem precisar de outra tecla.
- **`/`, Tab e Enter continuam esperando** uma leitura por nível: completar um nome
  ou confirmar um caminho precisa da resposta na hora, e foi a pessoa quem pediu.
  No Enter, o cursor vai até o fim do caminho digitado.

## O que continua síncrono

Igual ao rclone: F3, completar no Treespec, Enter num caminho digitado (uma
leitura por nível), abrir o Prune e o viewer.

## Testes

Sem rede:
- `Mount::reads_slowly` com linhas reais de `/proc/mounts`: SMB por drvfs (UNC),
  `/mnt/c` (9p local), cifs, nfs, rclone, ext4;
- a decodificação dos escapes octais, com escape incompleto ou inválido mantido
  como texto;
- Treespec numa árvore lenta (`Tree::slow`): digitar não lê, o cursor para na pasta
  pedida e segue o texto quando a leitura chega; Enter vai até o fim.
