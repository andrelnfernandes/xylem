# Xylem's layout catalog

*[Português](README.pt-BR.md)*

This folder holds the layouts built into Xylem: each `.xlayout.toml` describes a fixed-width or multi-record file format (CNAB, SPED...), and Xylem's viewer then shows those files as a table, with a sheet per kind of record, the checks of the totals and the meaning of the codes. The format is described in the manual, in the topic [Layouts](../docs/manual/layouts.md).

Contributing a layout needs no Rust: a TOML file in this folder is enough. `build.rs` builds every `.xlayout.toml` here into the program.

## The catalog's rule: public manuals only

A layout joins the catalog once it has been checked, field by field, against a manual anyone can download, and that manual is named in the file itself (`spec_source`, `spec_version`, `spec_url`). That way any user can check a position and point to a mistake precisely.

Layouts whose manual is not public (usually those a bank or a company hands to its customers, under an agreement or a registration) stay out of the catalog. Whoever uses them can keep them in their own folder, `~/.config/xylem/layouts/`, where Xylem uses them as it does the catalog's. If the owner of the manual allows the positions to be published, the contribution is welcome, with its source.

## What is here

| File | Manual | Status |
|---|---|---|
| `br/bancario/cnab240-febraban.xlayout.toml` | FEBRABAN, Layout Padrão 240 posições, version 11.0 (2026-09-11) | checked: file header and trailer, payments (A, B, C, Z), collection slips (J, J-52), collection (P, Q, R, T, U) |
| `br/fiscal/sped-generico.xlayout.toml` | Receita Federal, Guia Prático da EFD ICMS/IPI, version 3.2.2 (2026-02-11) | checked: opening and closing of blocks, 9900, 9990, 9999 and their counting rules |

### What CNAB 240 still lacks (public manual, to do)

- **J-52 for Pix:** the same positions as J-52 up to 131 and, from 132 to 240, the Pix key and the TXID. The manual tells them apart by the batch header's form of entry, whose code still has to be checked; until then, the layout shows these lines as an ordinary J-52.
- The manual's other services: payment of taxes (O, N, W), debit to account (debit A, B, C), statement for reconciliation (E), electronic payment slip (G, H), cheque custody, cash management, vendor, compror, payroll loans.
- The optional segments X (split payment of the tax reform) and Y (Y-01 to Y-56) of collection.

## Planned, with a public manual

These formats have an official, published specification, and anyone can write them from it. Before starting, check that the manual is still available and which version is current.

| Format | Published by |
|---|---|
| SPED EFD ICMS/IPI in full, EFD-Contribuições, ECD, ECF | Receita Federal (sped.rfb.gov.br) |
| Sintegra (Convênio ICMS 57/95) | CONFAZ |
| AFD (Portaria MTP 671/2021) and AFD/AFDT/ACJEF (Portaria 1.510/2009) | Ministério do Trabalho |
| SEFIP | Caixa Econômica Federal |
| FEBRABAN 150 positions (collection of bills) | FEBRABAN |
| B3 COTAHIST (historical quotes) | B3 |
| IRS Publication 1220 (FIRE) | IRS, United States |
| NOAA GHCN-Daily | NOAA, United States |

## For the community: manuals that are not public

The project does not include these layouts because their manuals are usually handed by the bank or the company to its customers, and could not be checked against an open source. If you have the manual and may share the positions, open an issue or a PR saying where they come from.

| Format | Note |
|---|---|
| CNAB 400 of each bank (Itaú, Bradesco, Banco do Brasil, Santander, Caixa, Sicoob, Sicredi...) | each bank has its own; there is no FEBRABAN standard of 400 positions |
| CNAB 240 with each bank's particulars | the FEBRABAN standard is already here; each bank's differences (fields for the bank's use, occurrence codes) come in its own manual |
| Statements of card acquirers (Cielo, Rede, Getnet, Stone...) | layouts given to merchants |
| EDI Proceda (NOTAFIS, CONEMB, OCOREN, DOCCOB, PREFAT) | to check whether a public version exists |
| Unimed's PTU interchange | given to the cooperatives |
| NACHA, IATA SSIM | the specifications are sold by their bodies |

## How to contribute

1. **Try it on your machine:** copy the file to `~/.config/xylem/layouts/` (`%APPDATA%\xylem\layouts\` on Windows). Open a sample file with `V`, press `K` and pick the layout; `i` shows the fields and the checks.
2. **Use made-up data:** never attach real files of customers, accounts or tax documents to an issue or PR.
3. **Name the source:** fill in `spec_source`, `spec_version` and `spec_url`, and put the manual's note code in each field's description when there is one (for example `(G001)`). Use `status = "verified"` only once every field has been checked against the manual.
4. **The `id` is the path:** a file at `br/fiscal/sintegra.xlayout.toml` has `id = "br/fiscal/sintegra"`.
5. **A specific `[detect]`:** the catalog is used on its own, so detection must recognize the format by its first line without mistaking other text files for it (use the line's length and more than one condition).
6. **Run the tests:** `cargo test catalog` checks that every layout in this folder is valid, covers its positions with no gaps or overlaps, and has the metadata required.

To point out a mistake in a layout, open an issue with the record, the field, the position it has and the one in the manual (with the version and the page).
