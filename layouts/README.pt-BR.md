# Catálogo de layouts do Xylem

*[English](README.md)*

Esta pasta guarda os layouts que o Xylem traz embutidos: cada `.xlayout.toml` descreve um formato de arquivo de largura fixa ou multi-registro (CNAB, SPED...), e o visualizador do Xylem passa a mostrar esses arquivos como tabela, com uma aba por tipo de registro, a conferência dos totais e o significado dos códigos. O formato está descrito no manual, no tópico [Layouts](../docs/manual/pt/layouts.md).

Contribuir com um layout não exige Rust: basta um arquivo TOML nesta pasta. O `build.rs` embute no programa todos os `.xlayout.toml` daqui.

## Regra do catálogo: só manual público

Um layout entra no catálogo quando foi conferido, campo a campo, contra um manual que qualquer pessoa pode baixar, e esse manual é citado no próprio arquivo (`spec_source`, `spec_version`, `spec_url`). Assim qualquer usuário pode verificar uma posição e apontar o erro com precisão.

Layouts cujo manual não é público (em geral, os entregues por um banco ou uma empresa aos seus clientes, mediante convênio ou cadastro) ficam fora do catálogo. Quem os usa pode mantê-los na própria pasta, `~/.config/xylem/layouts/`, onde o Xylem os usa como os do catálogo. Se o dono do manual permitir publicar as posições, a contribuição é bem-vinda, com a fonte indicada.

## O que já está aqui

| Arquivo | Manual | Situação |
|---|---|---|
| `br/bancario/cnab240-febraban.xlayout.toml` | FEBRABAN, Layout Padrão 240 posições, versão 11.0 (11/09/2026) | conferido: header e trailer de arquivo, pagamentos (A, B, C, Z), títulos (J, J-52), cobrança (P, Q, R, T, U) |
| `br/fiscal/sped-generico.xlayout.toml` | Receita Federal, Guia Prático da EFD ICMS/IPI, versão 3.2.2 (11/02/2026) | conferido: abertura e encerramento de blocos, 9900, 9990, 9999 e as suas regras de contagem |

### O que falta no CNAB 240 (manual público, a fazer)

- **J-52 para o Pix:** tem as mesmas posições do J-52 até a 131 e, de 132 a 240, a chave Pix e o TXID. O manual o distingue pela forma de lançamento do header de lote, cujo código ainda precisa ser conferido; até lá, o layout mostra essas linhas como J-52 comum.
- Os outros serviços do manual: pagamento de tributos (O, N, W), débito em conta (A, B, C de débito), extrato para conciliação (E), boleto de pagamento eletrônico (G, H), custódia de cheques, gestão de caixa, vendor, compror, consignação.
- Os segmentos opcionais X (split payment da reforma tributária) e Y (Y-01 a Y-56) da cobrança.

## Planejados, com manual público

Estes formatos têm especificação oficial publicada e podem ser feitos por qualquer pessoa a partir dela. Antes de começar, confira se o manual continua disponível e qual é a versão atual.

| Formato | Quem publica |
|---|---|
| SPED EFD ICMS/IPI completo, EFD-Contribuições, ECD, ECF | Receita Federal (sped.rfb.gov.br) |
| Sintegra (Convênio ICMS 57/95) | CONFAZ |
| AFD (Portaria MTP 671/2021) e AFD/AFDT/ACJEF (Portaria 1.510/2009) | Ministério do Trabalho |
| SEFIP | Caixa Econômica Federal |
| FEBRABAN 150 posições (arrecadação) | FEBRABAN |
| B3 COTAHIST (série histórica de cotações) | B3 |
| IRS Publication 1220 (FIRE) | IRS, Estados Unidos |
| NOAA GHCN-Daily | NOAA, Estados Unidos |

## Para a comunidade: manuais que não são públicos

O projeto não inclui estes layouts porque os seus manuais costumam ser distribuídos pelo banco ou pela empresa aos clientes, e não puderam ser conferidos com uma fonte aberta. Se você tem o manual e pode compartilhar as posições, abra uma issue ou um PR dizendo de onde elas vêm.

| Formato | Observação |
|---|---|
| CNAB 400 de cada banco (Itaú, Bradesco, Banco do Brasil, Santander, Caixa, Sicoob, Sicredi...) | cada banco tem o seu; não há um padrão FEBRABAN de 400 posições |
| CNAB 240 com as particularidades de cada banco | o padrão FEBRABAN já está aqui; as diferenças de cada banco (campos de uso do banco, códigos de ocorrência) vêm no manual de cada um |
| Extratos de adquirentes (Cielo, Rede, Getnet, Stone...) | layouts entregues aos estabelecimentos |
| EDI Proceda (NOTAFIS, CONEMB, OCOREN, DOCCOB, PREFAT) | a verificar se há versão pública |
| Intercâmbio Unimed PTU | distribuído às cooperativas |
| NACHA, IATA SSIM | as especificações são vendidas pelas entidades |

## Como contribuir

1. **Teste na sua máquina:** copie o arquivo para `~/.config/xylem/layouts/` (`%APPDATA%\xylem\layouts\` no Windows). Abra um arquivo de exemplo com `V`, aperte `K` e escolha o layout; `i` mostra os campos e as conferências.
2. **Use dados fictícios:** nunca anexe arquivos reais de clientes, contas ou documentos fiscais a uma issue ou PR.
3. **Cite a fonte:** preencha `spec_source`, `spec_version` e `spec_url`, e ponha na descrição de cada campo o código de nota do manual, quando houver (por exemplo, `(G001)`). Use `status = "verified"` só depois de conferir todos os campos com o manual.
4. **O `id` é o caminho:** um arquivo em `br/fiscal/sintegra.xlayout.toml` tem `id = "br/fiscal/sintegra"`.
5. **Um `[detect]` específico:** o catálogo é usado sozinho, então a detecção precisa reconhecer o formato pela primeira linha sem confundi-lo com outros arquivos de texto (use o comprimento da linha e mais de uma condição).
6. **Rode os testes:** `cargo test catalog` confere que todo layout desta pasta é válido, cobre as posições sem buracos nem sobreposições e tem os metadados exigidos.

Para apontar um erro num layout, abra uma issue com o registro, o campo, a posição atual e a posição no manual (com a versão e a página).
