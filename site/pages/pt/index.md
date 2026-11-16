# Xylem File Manager

<div class="xylem-hero">
<p class="xylem-hero-title">O disco inteiro, <em>uma só árvore</em></p>
<p class="lead">Um gerenciador de arquivos para o terminal, com a tela e as teclas do XTreeGold e do ZTreeWin. Ele lê diretórios para a memória, permitindo que uma só lista reúna arquivos de várias pastas e um único comando aja sobre todos os arquivos marcados.</p>
<div class="buttons">
<a class="button primary" href="start.md">Instalar</a>
<a class="button" href="https://github.com/andrelnfernandes/xylem">
<svg viewBox="0 0 16 16" aria-hidden="true"><path d="M8 0C3.58 0 0 3.58 0 8c0 3.54 2.29 6.53 5.47 7.59.4.07.55-.17.55-.38 0-.19-.01-.82-.01-1.49-2.01.37-2.53-.49-2.69-.94-.09-.23-.48-.94-.82-1.13-.28-.15-.68-.52-.01-.53.63-.01 1.08.58 1.23.82.72 1.21 1.87.87 2.33.66.07-.52.28-.87.51-1.07-1.78-.2-3.64-.89-3.64-3.95 0-.87.31-1.59.82-2.15-.08-.2-.36-1.02.08-2.12 0 0 .67-.21 2.2.82.64-.18 1.32-.27 2-.27.68 0 1.36.09 2 .27 1.53-1.04 2.2-.82 2.2-.82.44 1.1.16 1.92.08 2.12.51.56.82 1.27.82 2.15 0 3.07-1.87 3.75-3.65 3.95.29.25.54.73.54 1.48 0 1.07-.01 1.93-.01 2.2 0 .21.15.46.55.38A8.013 8.013 0 0016 8c0-4.42-3.58-8-8-8z"/></svg>
Código no GitHub</a>
</div>
<p class="note">Livre e de código aberto (MIT) · Linux x86_64 e ARM64 · Windows (experimental)</p>
<div class="window demo">
<div class="bar"><i></i><i></i><i></i><span>xy ~/projetos</span></div>
<img src="ASSETS/screenshots/demo-nord.gif" width="1104" height="772"
alt="O Xylem em uso: o terminal embutido com a árvore acompanhando um cd, arquivos marcados num Branch e digitados num comando com Alt+O">
</div>
</div>

## O que ele faz

<div class="features">
<div class="feature">
<h3>Lista arquivos entre pastas</h3>
<p>Branch lista um diretório com tudo o que está abaixo dele, e Showall lista tudo o que foi lido até agora. Marque arquivos ali e copie, mova, exclua ou renomeie todos juntos. <a href="overview.md">O que é o Xylem</a></p>
</div>
<div class="feature">
<h3>Mostra arquivos de dados como tabelas</h3>
<p>SQLite, planilhas, CSV, JSON Lines, DBF e Parquet abrem no visualizador como linhas e colunas, filtrados com uma condição SQL <code>WHERE</code>. Arquivos SQLite também aceitam consultas completas com <code>JOIN</code>. <a href="howto-table-sql.md">Como fazer</a></p>
</div>
<div class="feature">
<h3>Filtra a lista de arquivos com SQL</h3>
<p><code>ext = 'jpg' AND size &gt; 10000000</code> mantém os arquivos que atendem à condição. <a href="howto-sql-filter.md">Como fazer</a></p>
</div>
<div class="feature">
<h3>Funciona em outras máquinas</h3>
<p>Servidores via SSH e contêineres pelo <code>docker exec</code> aparecem como uma árvore, com cópia, movimentação, renomeação e exclusão. Google Drive, S3 e outros armazenamentos vêm pelo rclone. <a href="howto-ssh-copy.md">Como fazer</a></p>
</div>
<div class="feature">
<h3>Descobre o que ocupa espaço</h3>
<p>Usage ordena as pastas e arquivos de um ramo por tamanho. Duplicates encontra arquivos e pastas com o mesmo conteúdo; no armazenamento rclone ele compara os hashes mantidos pelo serviço sem baixar nada. <a href="howto-disk-space.md">Como fazer</a></p>
</div>
<div class="feature">
<h3>Tem um terminal embutido</h3>
<p>A árvore acompanha o <code>cd</code> da shell, e Alt+O digita os arquivos marcados na linha de comando. <a href="howto-terminal-tagged.md">Como fazer</a></p>
</div>
<div class="feature">
<h3>Trata arquivos como hostis</h3>
<p>Ver um arquivo não executa nada dele, e cada leitor tem um limite de tamanho. <a href="security.md">Segurança</a></p>
</div>
<div class="feature">
<h3>Temas, teclas e idiomas</h3>
<p>Doze temas, teclas definidas em arquivo de configuração e ajuda em inglês, português, espanhol, francês, alemão e chinês. <a href="configuration.md">Configuração</a></p>
</div>
</div>
