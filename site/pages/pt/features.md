# Recursos

<p class="xylem-lead">Tudo isso, em detalhes</p>
<p class="section-lead">A ajuda (<kbd>F1</kbd>) explica cada um, em seis línguas.</p>
<div class="features">
<div class="feature">
<h3>Uma árvore, muitos arquivos</h3>
<p>Uma árvore de diretórios logada na memória com a janela de arquivos embaixo, Branch e
Showall para listar os arquivos de um ramo inteiro ou de tudo o que foi logado, e tags
sobre as quais agem as operações em grupo.</p>
</div>
<div class="feature">
<h3>Operações com arquivos</h3>
<p>Copiar e mover com ou sem os caminhos, com progresso e detalhes dos conflitos. Apagar
para a lixeira com Oops para restaurar, Prune, renomear com máscaras, permissões e
datas, Compare entre duas janelas.</p>
</div>
<div class="feature">
<h3>Achar coisas</h3>
<p>Filespec por nome, data e tamanho, Treespec para digitar um caminho, bookmarks, e
<kbd>Ctrl</kbd>+<kbd>S</kbd> para buscar texto nos arquivos marcados.</p>
</div>
<div class="feature">
<h3>Olhar dentro e editar</h3>
<p>Um visualizador de texto e hex com busca, quebra de linha, numeração e cores de sintaxe,
um modo tabela com filtro SQL para planilhas, CSV, JSON Lines, DBF, SQLite e Parquet,
arquivos Markdown renderizados, além do Autoview ao lado da lista. <kbd>E</kbd> abre seu editor de texto ou um editor
hexadecimal (<code>hexedit</code>, <code>heh</code>, <code>bvi</code>, <code>dhex</code>)
direto dentro da moldura e com o tema do Xylem — ou desativa a edição binária no <kbd>F12</kbd>.</p>
</div>
<div class="feature">
<h3>Batch sobre registros</h3>
<p><kbd>Ctrl</kbd>+<kbd>B</kbd> transforma arquivos marcados, os registros de uma tabela ou as
linhas de um texto em script, JSON, CSV, XML, HTML, SQL ou um banco SQLite, com escapes que
mantêm dados hostis no lugar.</p>
</div>
<div class="feature">
<h3>Duplicados</h3>
<p><kbd>Alt</kbd>+<kbd>Y</kbd> acha arquivos e pastas inteiras com o mesmo conteúdo, sem baixar
nada de um Google Drive montado pelo rclone, e sempre deixa uma cópia.</p>
</div>
<div class="feature">
<h3>Arquivos compactados</h3>
<p>ZIP, 7z e TAR abrem como diretórios: navegue, veja e copie de dentro deles. Também dá
para criar ZIP e TAR.</p>
</div>
<div class="feature">
<h3>Terminal embutido</h3>
<p><kbd>Alt</kbd>+<kbd>X</kbd> abre um shell cujo <code>cd</code> leva a árvore junto;
<kbd>Alt</kbd>+<kbd>O</kbd> digita os arquivos marcados no prompt.</p>
</div>
<div class="feature">
<h3>Remoto e nuvem</h3>
<p>Navegue e copie de máquinas via SSH, com o terminal de lá. Google Drive, Amazon S3 e
uns 70 outros serviços via <a href="https://rclone.org">rclone</a>.</p>
</div>
<div class="feature">
<h3>Git e mais</h3>
<p>Um menu Git opcional com letras de status na lista de arquivos, a área de
transferência, uma tela de configuração (<kbd>F12</kbd>) e doze temas.</p>
</div>
</div>


## Temas

<p class="xylem-lead">Deixe do seu jeito</p>
<p class="section-lead">Escolha um tema com <kbd>F4</kbd>, ou defina <code>theme</code> no
arquivo de configuração. Cada cor também pode ser trocada uma a uma.</p>
<div class="themes">
<a href="ASSETS/screenshots/theme-nord.png" aria-current="true">nord</a>
<a href="ASSETS/screenshots/theme-slate.png">slate</a>
<a href="ASSETS/screenshots/theme-graphite.png">graphite</a>
<a href="ASSETS/screenshots/theme-daylight.png">daylight</a>
<a href="ASSETS/screenshots/theme-vivid.png">vivid</a>
<a href="ASSETS/screenshots/theme-dark.png">dark</a>
<a href="ASSETS/screenshots/theme-light.png">light</a>
<a href="ASSETS/screenshots/theme-gruvbox.png">gruvbox</a>
<a href="ASSETS/screenshots/theme-solarized.png">solarized</a>
<a href="ASSETS/screenshots/theme-mono.png">mono</a>
<a href="ASSETS/screenshots/theme-norton.png">norton</a>
<a href="ASSETS/screenshots/tree.png">classic</a>
</div>
<div class="window theme-view"><img src="ASSETS/screenshots/theme-nord.png" width="1104" height="772" loading="lazy" alt="A mesma tela no tema nord" data-alt="A mesma tela no tema {}"></div>


## Uso do disco

<p class="xylem-lead">Para onde foi o espaço?</p>
<p class="section-lead"><kbd>Alt</kbd>+<kbd>W</kbd> lista as pastas e os arquivos de um ramo pelo espaço que
ocupam, do maior para o menor, com a parte de cada um e, no painel ao lado, a data. Desça
pasta por pasta até o que enche o disco: um backup antigo, uma ISO esquecida, um cache que
cresceu. Um disco por vez, hard links contados uma vez e o espaço em disco como o
<code>du</code> conta.</p>
<div class="window theme-view"><img src="ASSETS/screenshots/usage-vivid.png" width="1104" height="772" loading="lazy" alt="A visão Usage de uma pasta pessoal: um arquivo de backup antigo de 72 MiB no topo com 37%, depois as pastas de projetos, vídeos e downloads"></div>
<p class="section-lead after">Na árvore, <kbd>Alt</kbd>+<kbd>K</kbd> → <strong>Percent</strong> mostra as mesmas partes
ao lado de cada pasta. Apontado para um Google Drive ou um bucket S3 montado com o
<a href="https://rclone.org">rclone</a>, mostra também o que os enche.</p>

