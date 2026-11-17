# Plano: certificados e chaves no viewer

Origem: conversa de 2026-11-16 (veja [viewer-logs.md](viewer-logs.md)).
**Spec proposto, ainda não aprovado.**

## Objetivo

Um certificado, uma corrente, um pedido de certificado (CSR) ou uma chave abre
como um relatório: de quem é, quem emitiu, até quando vale, para que serve e se
usa algo fraco. Uma chave privada **nunca tem o conteúdo mostrado no
relatório**, e o Autoview deixa de mostrá-la em texto só por passar sobre o
arquivo na árvore.

Hoje um PEM abre como texto em base64, e um DER abre em hex.

## Formatos

- **X.509:** certificado em PEM ou DER (`.pem`, `.crt`, `.cer`, `.der`), e um
  arquivo com vários (corrente, bundle de CAs como o `ca-certificates.crt`).
- **CSR** (PKCS#10, `.csr`) e **CRL** (`.crl`).
- **Chaves:** PKCS#1, PKCS#8 (inclusive cifrada), SEC1 (EC), a chave privada e a
  pública do OpenSSH, `authorized_keys` e `known_hosts`.
- **PKCS#7** (`.p7b`, `.p7c`): os certificados de dentro.
- Fora: PKCS#12 (`.p12`, `.pfx`) e JKS, que pedem senha (só o tipo é dito), e
  chaves PGP.

## Detecção

- PEM: um `-----BEGIN ...-----` nas primeiras 200 linhas, em qualquer arquivo
  de texto até 1 MiB. Um arquivo com texto e blocos misturados (comentários
  antes de cada certificado) vale: o relatório lista cada bloco e diz quantas
  linhas ficaram fora deles.
- DER: pela extensão (`.der`, `.cer`, `.crt`, `.crl`, `.csr`, `.p7b`) e um
  `SEQUENCE` (`0x30`) cujo tamanho bate com o do arquivo.
- OpenSSH: pelo armor (`OPENSSH PRIVATE KEY`) e pelas linhas `ssh-ed25519 AAAA…`
  e `ecdsa-sha2-…` em `.pub`, `authorized_keys` e `known_hosts`.

## O relatório

Montado pelo `markdown::Builder` com eventos (etapa 1 de
[viewer-office.md](viewer-office.md)). `T` mostra a fonte e `H` mostra o hex.

**Certificado**, um bloco por certificado:

- Subject e issuer, por extenso, e se é **autoassinado**.
- Validade: início e fim, com quantos dias faltam. **Vencido** ou **ainda não
  válido** aparecem em destaque, e o que vence em até 30 dias também.
- Chave pública: algoritmo e tamanho (RSA 2048, EC P-256, Ed25519).
- Assinatura: algoritmo. **SHA-1, MD5 e RSA abaixo de 2048** aparecem em
  destaque como fracos.
- Nomes alternativos (SAN), uso da chave, uso estendido (servidor, cliente,
  assinatura de código), restrições básicas (CA ou não, `pathlen`), SKI e AKI.
- Endereços de CRL, OCSP e AIA, **mostrados e nunca buscados**.
- Fingerprints SHA-256 e SHA-1 (o `sha2` já é dependência; o SHA-1 é só para
  comparar com ferramentas antigas e vem marcado assim).
- Extensões que o leitor não conhece: o OID e se é crítica. Uma extensão
  crítica desconhecida fica em destaque.

**Corrente:** a ordem é conferida **só pelos nomes** (o issuer de um é o subject
do seguinte, e o AKI bate com o SKI). O relatório diz claramente que **as
assinaturas não são verificadas**: verificar exigiria criptografia em C ou
assembly (`ring`, `aws-lc`), o que vai contra o princípio de parsers em Rust.

**Chave privada:**

- Só o tipo, o tamanho ou a curva, se está cifrada (e com que algoritmo, no
  PKCS#8 e no OpenSSH), o comentário do OpenSSH e o fingerprint da chave pública
  correspondente.
- **Os números da chave nunca vão para o relatório**, nem para a busca, nem para
  o Ctrl+B.

**Chave pública, `authorized_keys` e `known_hosts`:** uma linha por chave,
com tipo, tamanho, fingerprint SHA-256 (o formato do `ssh-keygen -l`),
comentário e opções (`from=`, `command=`). Um `command=` aparece em destaque, porque
muda o que a chave permite. Em `known_hosts`, o host aparece em claro ou como
`|1|…` quando está com hash.

## Chave privada no Autoview

- Hoje o Autoview mostra o texto de `~/.ssh/id_ed25519` só por passar sobre
  ele, e isso aparece numa tela compartilhada ou gravada.
- Proposta (?1): no Autoview, todo arquivo detectado como chave privada mostra
  só o relatório (tipo e fingerprint), sem o texto. Com `V`, o arquivo abre no
  relatório, e `T` mostra o texto, como hoje.

## Leitor

- **`x509-parser`** (rusticata), com as features de PEM, e sem a de verificação,
  que traz o `ring`. Rust puro, testado com fuzzing (OSS-Fuzz). Traz `der-parser`
  e `asn1-rs`.
- **Chaves:** o próprio `x509-parser` para PKCS#8 e SPKI. PKCS#1, SEC1 e o
  formato do OpenSSH com um leitor próprio e pequeno, só do que o relatório
  mostra (tipo, tamanho, se está cifrada). O `ssh-key` traria criptografia
  inteira para ler três campos.
- Dependências novas: `x509-parser` e o que ele traz, com revisão do `unsafe`
  (princípio 7 de [viewers.md](viewers.md)).

## Segurança

- **ASN.1:** o `asn1-rs` tem limite de recursão. Mesmo assim, profundidade e
  tamanho são conferidos antes, numa passada sem recursão pelos cabeçalhos
  TLV, que recusa o bloco se passar de 64 níveis ou se um tamanho declarado
  passar do que resta.
- **Tetos:** 1 MiB por arquivo para procurar PEM, 64 KiB por bloco, 1.000 blocos
  (o bundle do sistema tem uns 150), 256 extensões por certificado e 1.000
  nomes num SAN. Acima disso, corta e diz.
- **O que chega à tela:** todo texto de nome (subject, SAN, comentário) passa pela
  limpeza do texto. Num certificado, um nome pode ter qualquer byte, inclusive
  `\0`, e o `\0` dentro de um CN já serviu para enganar navegadores
  (CVE-2009-2408). Ele aparece como `·`, e o nome fica em destaque.
- **IDN:** um nome em punycode (`xn--`) aparece como está, e ao lado a forma
  Unicode marcada, para que um homógrafo não passe por outro nome.
- **Nada sai da máquina:** nenhum endereço de CRL, OCSP ou AIA é buscado.
- **Pânico** vira texto ou hex, por `panic_guard::catch`.
- proptest: bytes aleatórios dentro de um armor PEM, DER com tamanhos trocados
  e certificados reais com campos mudados nunca dão pânico. Nenhum número de
  chave privada aparece no relatório (o teste procura os bytes da chave no
  texto gerado).

## Etapas (um PR cada)

1. **Certificado X.509** em PEM e DER, inclusive a corrente.
2. **Chaves** (PKCS#1, PKCS#8, SEC1 e OpenSSH) e o comportamento no Autoview.
3. **`authorized_keys` e `known_hosts`.**
4. **CSR, CRL e PKCS#7.**

## Questões abertas

| # | Questão | Proposta |
|---|---|---|
| ?1 | O Autoview esconde o texto de uma chave privada? | sim, mostra só o relatório |
| ?2 | Dias para o destaque de "vence logo" | 30, e configurável depois se houver pedido |
| ?3 | Verificar assinaturas | não: só os nomes da corrente, dito claramente |

## Para depois

- PKCS#12 com senha pedida na hora (exigiria a decifração, que hoje não há em
  Rust puro e auditado).
- Chaves PGP.
- Conferir uma chave de `known_hosts` contra a que o servidor mostrou numa
  conexão SSH já aberta pelo Xylem.
