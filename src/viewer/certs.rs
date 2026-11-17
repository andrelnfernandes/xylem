//! Certificates and keys (docs/plans/viewer-certs.md): X.509 certificates
//! and chains, requests (CSR), revocation lists, private and public keys,
//! and OpenSSH's keys, `authorized_keys` and `known_hosts`, as a report.
//!
//! The numbers of a private key never go into the report, nor into a
//! search. A chain is checked only by its names: no signature is verified
//! here (that would take cryptography in C or assembly), and the report says
//! so. Nothing is fetched: the addresses of a CRL, OCSP or AIA are shown.

use std::path::Path;
use std::time::Duration;

use sha2::{Digest, Sha256};
use x509_parser::prelude::*;

use super::doc_events::{DocEvents, Look};
use super::markdown::{self, Doc};

/// Most of a file searched for PEM blocks, most blocks, and most of one.
const MAX_SEARCH: usize = 1 << 20;
const MAX_BLOCKS: usize = 1000;
const MAX_BLOCK: usize = 64 << 10;
/// Most levels of ASN.1 nesting.
const MAX_DER_DEPTH: usize = 64;
const MAX_NAMES: usize = 1000;
const MAX_EVENTS: usize = 1_000_000;
/// Days before the end of a certificate's validity from which it is marked.
const SOON: i64 = 30;

/// A certificate or a key, by its name and its contents.
pub fn detect(path: &Path, data: &[u8]) -> bool {
    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(str::to_ascii_lowercase)
        .unwrap_or_default();
    let head = &data[..data.len().min(MAX_SEARCH)];
    // PEM only in a file named for it, or with no extension (`id_ed25519`):
    // source code and documents often carry a PEM block as an example.
    let pem_name = ext.is_empty()
        || matches!(
            ext.as_str(),
            "pem"
                | "crt"
                | "cer"
                | "key"
                | "csr"
                | "crl"
                | "p7b"
                | "p7c"
                | "pub"
                | "p8"
                | "cert"
                | "ca-bundle"
        );
    if pem_name && pem_start(head).is_some() {
        return true;
    }
    if matches!(
        ext.as_str(),
        "der" | "cer" | "crt" | "crl" | "csr" | "p8" | "key"
    ) && der_whole(data)
    {
        return true;
    }
    if name.starts_with("authorized_keys") || name.starts_with("known_hosts") {
        return true;
    }
    ext == "pub" && ssh_lines(head).next().is_some()
}

/// Where the first PEM block starts, in the first lines.
fn pem_start(head: &[u8]) -> Option<usize> {
    let text = std::str::from_utf8(head).ok().or_else(|| {
        // Cut in the middle of a character at the end of the head.
        let mut end = head.len();
        while end > 0 && std::str::from_utf8(&head[..end]).is_err() {
            end -= 1;
        }
        std::str::from_utf8(&head[..end]).ok()
    })?;
    let mut at = 0;
    for line in text.split_inclusive('\n').take(200) {
        if line.starts_with("-----BEGIN ") {
            return Some(at);
        }
        at += line.len();
    }
    None
}

/// The bytes are one DER value, whole, with no more than [`MAX_DER_DEPTH`]
/// levels, checked with no recursion before any parser sees them.
pub fn der_whole(data: &[u8]) -> bool {
    der_depth(data).is_some_and(|(len, depth)| len == data.len() && depth <= MAX_DER_DEPTH)
}

/// The length of the first DER value of `data` and how deep it nests, or
/// `None` if a length runs past the end.
fn der_depth(data: &[u8]) -> Option<(usize, usize)> {
    if data.first() != Some(&0x30) {
        return None;
    }
    let mut ends: Vec<usize> = Vec::new();
    let mut most = 0;
    let mut i = 0;
    let mut first_end = None;
    loop {
        while ends.last() == Some(&i) {
            ends.pop();
        }
        if i >= data.len() || (first_end.is_some() && ends.is_empty()) {
            break;
        }
        let tag = *data.get(i)?;
        i += 1;
        if tag & 0x1f == 0x1f {
            // A long tag number.
            let mut n = 0;
            while data.get(i)? & 0x80 != 0 {
                i += 1;
                n += 1;
                if n > 4 {
                    return None;
                }
            }
            i += 1;
        }
        let first = *data.get(i)?;
        i += 1;
        let len = if first < 0x80 {
            usize::from(first)
        } else {
            let n = usize::from(first & 0x7f);
            // No indefinite length in DER, and no length over 4 bytes.
            if n == 0 || n > 4 {
                return None;
            }
            let mut len = 0usize;
            for _ in 0..n {
                len = len.checked_mul(256)? + usize::from(*data.get(i)?);
                i += 1;
            }
            len
        };
        let end = i.checked_add(len)?;
        if end > data.len() || ends.last().is_some_and(|&e| end > e) {
            return None;
        }
        if first_end.is_none() {
            first_end = Some(end);
        }
        if tag & 0x20 != 0 {
            ends.push(end);
            most = most.max(ends.len());
            if most > MAX_DER_DEPTH {
                return Some((end, most));
            }
        } else {
            i = end;
        }
    }
    Some((first_end?, most))
}

/// The lines of OpenSSH keys: `[options] type base64 [comment]`.
fn ssh_lines(text: &[u8]) -> impl Iterator<Item = &str> {
    std::str::from_utf8(text)
        .unwrap_or("")
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .filter(|l| l.split_whitespace().any(is_ssh_type))
}

fn is_ssh_type(word: &str) -> bool {
    word.starts_with("ssh-")
        || word.starts_with("ecdsa-sha2-")
        || word.starts_with("sk-ssh-")
        || word.starts_with("sk-ecdsa-")
}

fn name_of(oid: &x509_parser::der_parser::oid::Oid) -> String {
    x509_parser::objects::oid2sn(oid, x509_parser::objects::oid_registry())
        .map(str::to_string)
        .unwrap_or_else(|_| oid.to_id_string())
}

/// Text from the file for the screen: a `\0` (which once fooled browsers in
/// a CN, CVE-2009-2408) and any control character as `·`.
fn shown(s: &str) -> String {
    s.chars()
        .map(|c| if c.is_control() { '·' } else { c })
        .collect()
}

fn fingerprint(der: &[u8]) -> String {
    let hash = Sha256::digest(der);
    hash.iter()
        .map(|b| format!("{b:02X}"))
        .collect::<Vec<_>>()
        .join(":")
}

/// The bits of a big number written in DER (its leading zeros not counted).
fn bits(n: &[u8]) -> usize {
    let n: &[u8] = {
        let start = n.iter().position(|&b| b != 0).unwrap_or(n.len());
        &n[start..]
    };
    match n.first() {
        Some(&b) => (n.len() - 1) * 8 + (8 - b.leading_zeros() as usize),
        None => 0,
    }
}

/// The algorithm and size of a public key, and whether it is weak.
fn key_text(spki: &SubjectPublicKeyInfo) -> (String, bool) {
    let alg = name_of(&spki.algorithm.algorithm);
    match spki.parsed() {
        Ok(x509_parser::public_key::PublicKey::RSA(rsa)) => {
            let n = bits(rsa.modulus);
            (format!("RSA {n} bits"), n < 2048)
        }
        Ok(x509_parser::public_key::PublicKey::EC(_)) => {
            let curve = spki
                .algorithm
                .parameters
                .as_ref()
                .and_then(|p| p.as_oid().ok())
                .map(|o| name_of(&o))
                .unwrap_or_else(|| "?".into());
            (format!("EC {curve}"), false)
        }
        Ok(x509_parser::public_key::PublicKey::DSA(_)) => ("DSA".into(), true),
        _ => (alg, false),
    }
}

/// A signature algorithm that is weak today: MD2, MD5 or SHA-1.
fn weak_signature(name: &str) -> bool {
    let n = name.to_ascii_lowercase();
    n.contains("md5") || n.contains("md2") || n.contains("sha1") || n.contains("sha-1")
}

fn time_text(t: &ASN1Time) -> String {
    let ts = t.timestamp();
    chrono::DateTime::from_timestamp(ts, 0).map_or_else(
        || ts.to_string(),
        |d| d.format("%Y-%m-%d %H:%M:%S UTC").to_string(),
    )
}

/// A row of the report with `⚠` when it is a problem.
fn mark(d: &mut DocEvents, label: &str, value: &str, bad: bool) {
    d.start(pulldown_cmark::Tag::Paragraph);
    d.styled(&format!("{label}: "), Look::Strong);
    d.text(value);
    if bad {
        d.styled("  ⚠", Look::Strong);
    }
    d.end(pulldown_cmark::TagEnd::Paragraph);
}

/// What the report kept of each certificate, to check a chain by names.
struct Link {
    subject: Vec<u8>,
    issuer: Vec<u8>,
    ski: Option<Vec<u8>>,
    aki: Option<Vec<u8>>,
}

fn certificate(d: &mut DocEvents, der: &[u8], n: usize, now: i64) -> Result<Link, String> {
    let (_, cert) = X509Certificate::from_der(der)
        .map_err(|e| format!("a certificate that cannot be read ({e})"))?;
    let tbs = &cert.tbs_certificate;
    d.heading(2, &format!("Certificate {n}"));
    mark(d, "Subject", &shown(&tbs.subject.to_string()), false);
    let self_signed = tbs.subject.as_raw() == tbs.issuer.as_raw();
    mark(
        d,
        "Issuer",
        &if self_signed {
            format!("{} (self-signed)", shown(&tbs.issuer.to_string()))
        } else {
            shown(&tbs.issuer.to_string())
        },
        false,
    );
    mark(d, "Serial", &tbs.raw_serial_as_string(), false);
    let (from, to) = (
        tbs.validity.not_before.timestamp(),
        tbs.validity.not_after.timestamp(),
    );
    let days = (to - now).div_euclid(86_400);
    let (state, bad) = if now < from {
        ("not valid yet".to_string(), true)
    } else if now > to {
        (format!("expired {} days ago", -days), true)
    } else if days <= SOON {
        (format!("expires in {days} days"), true)
    } else {
        (format!("{days} days left"), false)
    };
    mark(
        d,
        "Valid",
        &format!(
            "{} to {} ({state})",
            time_text(&tbs.validity.not_before),
            time_text(&tbs.validity.not_after)
        ),
        bad,
    );
    let (key, weak_key) = key_text(&tbs.subject_pki);
    mark(d, "Public key", &key, weak_key);
    let sig = name_of(&cert.signature_algorithm.algorithm);
    mark(d, "Signature", &sig, weak_signature(&sig));

    let mut ski = None;
    let mut aki = None;
    let mut names: Vec<String> = Vec::new();
    for ext in tbs.extensions() {
        match ext.parsed_extension() {
            ParsedExtension::SubjectAlternativeName(san) => {
                for g in san.general_names.iter().take(MAX_NAMES) {
                    names.push(general_name(g));
                }
                if san.general_names.len() > MAX_NAMES {
                    names.push(format!("… {} names in all", san.general_names.len()));
                }
            }
            ParsedExtension::KeyUsage(k) => mark(d, "Key usage", &k.to_string(), false),
            ParsedExtension::ExtendedKeyUsage(e) => {
                let mut u = Vec::new();
                for (on, n) in [
                    (e.any, "any"),
                    (e.server_auth, "server"),
                    (e.client_auth, "client"),
                    (e.code_signing, "code signing"),
                    (e.email_protection, "e-mail"),
                    (e.time_stamping, "time stamping"),
                    (e.ocsp_signing, "OCSP signing"),
                ] {
                    if on {
                        u.push(n.to_string());
                    }
                }
                u.extend(e.other.iter().map(name_of));
                mark(d, "Extended key usage", &u.join(", "), false);
            }
            ParsedExtension::BasicConstraints(b) => {
                let text = match (b.ca, b.path_len_constraint) {
                    (true, Some(n)) => format!("a CA, path length {n}"),
                    (true, None) => "a CA".into(),
                    (false, _) => "not a CA".into(),
                };
                mark(d, "Basic constraints", &text, false);
            }
            ParsedExtension::SubjectKeyIdentifier(k) => ski = Some(k.0.to_vec()),
            ParsedExtension::AuthorityKeyIdentifier(a) => {
                aki = a.key_identifier.as_ref().map(|k| k.0.to_vec())
            }
            ParsedExtension::CRLDistributionPoints(points) => {
                let urls: Vec<String> = points
                    .points
                    .iter()
                    .filter_map(|p| match &p.distribution_point {
                        Some(DistributionPointName::FullName(n)) => {
                            Some(n.iter().map(general_name).collect::<Vec<_>>().join(", "))
                        }
                        _ => None,
                    })
                    .collect();
                mark(d, "CRL (not fetched)", &urls.join(", "), false);
            }
            ParsedExtension::AuthorityInfoAccess(aia) => {
                let list: Vec<String> = aia
                    .accessdescs
                    .iter()
                    .map(|a| {
                        format!(
                            "{}: {}",
                            name_of(&a.access_method),
                            general_name(&a.access_location)
                        )
                    })
                    .collect();
                mark(d, "Authority info (not fetched)", &list.join(", "), false);
            }
            ParsedExtension::UnsupportedExtension { .. } | ParsedExtension::ParseError { .. } => {
                mark(
                    d,
                    "Extension",
                    &format!(
                        "{}{}",
                        name_of(&ext.oid),
                        if ext.critical {
                            " (critical, not known here)"
                        } else {
                            ""
                        }
                    ),
                    ext.critical,
                );
            }
            _ => {}
        }
    }
    if !names.is_empty() {
        d.start(pulldown_cmark::Tag::Paragraph);
        d.styled("Names: ", Look::Strong);
        d.end(pulldown_cmark::TagEnd::Paragraph);
        d.start_list(false);
        for n in &names {
            d.item(n);
        }
        d.end_list(false);
    }
    mark(d, "SHA-256", &fingerprint(der), false);
    Ok(Link {
        subject: tbs.subject.as_raw().to_vec(),
        issuer: tbs.issuer.as_raw().to_vec(),
        ski,
        aki,
    })
}

/// A name of a certificate. A punycode name (`xn--`) shows as it is, so a
/// look-alike in another script cannot pass for another name.
fn general_name(g: &GeneralName) -> String {
    let s = match g {
        GeneralName::DNSName(n) => format!("DNS {n}"),
        GeneralName::RFC822Name(n) => format!("e-mail {n}"),
        GeneralName::URI(n) => format!("URI {n}"),
        GeneralName::IPAddress(ip) => match ip.len() {
            4 => format!("IP {}", std::net::Ipv4Addr::new(ip[0], ip[1], ip[2], ip[3])),
            16 => {
                let mut b = [0u8; 16];
                b.copy_from_slice(ip);
                format!("IP {}", std::net::Ipv6Addr::from(b))
            }
            _ => "IP ?".into(),
        },
        GeneralName::DirectoryName(n) => format!("directory {n}"),
        other => format!("{other:?}"),
    };
    let s = shown(&s);
    if s.contains("xn--") {
        format!("{s} (punycode: an international name)")
    } else {
        s
    }
}

/// A private key: its kind and size only. Its numbers never leave here.
fn private_key(d: &mut DocEvents, label: &str, der: &[u8]) {
    use x509_parser::der_parser::ber::BerObjectContent as C;
    use x509_parser::der_parser::der::parse_der;
    d.heading(2, "Private key");
    let parsed = parse_der(der).ok().map(|(_, o)| o);
    let seq = parsed.as_ref().and_then(|o| match &o.content {
        C::Sequence(items) => Some(items.clone()),
        _ => None,
    });
    let text = match (label, seq) {
        ("ENCRYPTED PRIVATE KEY", Some(items)) => {
            let alg = items.first().and_then(|a| match &a.content {
                C::Sequence(x) => x.first().and_then(|o| o.as_oid().ok()).map(|o| name_of(o)),
                _ => None,
            });
            format!("PKCS#8, encrypted ({})", alg.unwrap_or_else(|| "?".into()))
        }
        ("RSA PRIVATE KEY", Some(items)) => {
            let n = items
                .get(1)
                .and_then(|n| n.as_slice().ok())
                .map(bits)
                .unwrap_or(0);
            format!("RSA {n} bits (PKCS#1)")
        }
        ("EC PRIVATE KEY", Some(items)) => {
            let curve = items.iter().find_map(|o| match &o.content {
                C::Unknown(any) if any.tag().0 == 0 => parse_der(any.data)
                    .ok()
                    .and_then(|(_, x)| x.as_oid().ok().map(|o| name_of(o))),
                C::Tagged(_, _, inner) => inner.as_oid().ok().map(|o| name_of(o)),
                _ => None,
            });
            format!("EC {} (SEC1)", curve.unwrap_or_else(|| "?".into()))
        }
        (_, Some(items)) => {
            // PKCS#8: version, algorithm, the key.
            let alg = items.get(1).and_then(|a| match &a.content {
                C::Sequence(x) => Some(x.clone()),
                _ => None,
            });
            let oid = alg
                .as_ref()
                .and_then(|a| a.first())
                .and_then(|o| o.as_oid().ok())
                .map(|o| name_of(o));
            let curve = alg
                .as_ref()
                .and_then(|a| a.get(1))
                .and_then(|o| o.as_oid().ok())
                .map(|o| name_of(o));
            let rsa_bits = items
                .get(2)
                .and_then(|k| k.as_slice().ok())
                .and_then(|inner| parse_der(inner).ok())
                .and_then(|(_, o)| match o.content {
                    C::Sequence(x) => x.get(1).and_then(|n| n.as_slice().ok()).map(bits),
                    _ => None,
                });
            match (oid.as_deref(), rsa_bits, curve) {
                (Some("rsaEncryption"), Some(b), _) => format!("RSA {b} bits (PKCS#8)"),
                (Some(o), _, Some(c)) => format!("{o} {c} (PKCS#8)"),
                (Some(o), _, None) => format!("{o} (PKCS#8)"),
                _ => "of a kind not known here".into(),
            }
        }
        _ => "that cannot be read".into(),
    };
    mark(d, "Kind", &text, false);
    d.paragraph("Its numbers are not shown. Anyone who sees this file can use the key.");
}

/// The fields of an OpenSSH key blob: its type, and the rest as strings.
fn ssh_blob(blob: &[u8]) -> Option<(String, Vec<&[u8]>)> {
    let mut parts = Vec::new();
    let mut i = 0;
    while i + 4 <= blob.len() && parts.len() < 16 {
        let n = u32::from_be_bytes(blob[i..i + 4].try_into().ok()?) as usize;
        i += 4;
        let end = i.checked_add(n)?;
        parts.push(blob.get(i..end)?);
        i = end;
    }
    let kind = String::from_utf8_lossy(parts.first()?).into_owned();
    Some((kind, parts))
}

/// The kind and size of an OpenSSH public key blob, and its SHA-256
/// fingerprint as `ssh-keygen -l` writes it.
fn ssh_key(blob: &[u8]) -> (String, String) {
    let fp = format!(
        "SHA256:{}",
        data_encoding::BASE64_NOPAD.encode(&Sha256::digest(blob))
    );
    let kind = match ssh_blob(blob) {
        Some((k, parts)) if k == "ssh-rsa" => {
            format!("RSA {} bits", parts.get(2).map_or(0, |n| bits(n)))
        }
        Some((k, parts)) if k.starts_with("ecdsa-sha2-") => format!(
            "ECDSA {}",
            shown(&String::from_utf8_lossy(
                parts.get(1).copied().unwrap_or(b"?")
            ))
        ),
        Some((k, _)) if k == "ssh-ed25519" => "Ed25519 256 bits".into(),
        Some((k, _)) if k == "ssh-dss" => "DSA (weak)".into(),
        Some((k, _)) => shown(&k),
        None => "?".into(),
    };
    (kind, fp)
}

fn openssh_private(d: &mut DocEvents, body: &[u8]) {
    d.heading(2, "OpenSSH private key");
    let magic = b"openssh-key-v1\0";
    let Some(rest) = body.strip_prefix(magic) else {
        mark(d, "Kind", "that cannot be read", false);
        return;
    };
    // ciphername, kdfname, kdfoptions, then the number of keys and the
    // public key.
    let mut fields = Vec::new();
    let mut i = 0;
    while fields.len() < 3 && i + 4 <= rest.len() {
        let n = u32::from_be_bytes([rest[i], rest[i + 1], rest[i + 2], rest[i + 3]]) as usize;
        i += 4;
        let Some(f) = i.checked_add(n).and_then(|e| rest.get(i..e)) else {
            break;
        };
        fields.push(f);
        i += n;
    }
    let cipher = fields
        .first()
        .map_or("?".to_string(), |c| shown(&String::from_utf8_lossy(c)));
    mark(
        d,
        "Encrypted",
        &if cipher == "none" {
            "no".to_string()
        } else {
            format!("yes ({cipher})")
        },
        cipher == "none",
    );
    i += 4; // the number of keys
    if let Some(n) = rest
        .get(i..i + 4)
        .map(|b| u32::from_be_bytes([b[0], b[1], b[2], b[3]]) as usize)
        && let Some(blob) = rest.get(i + 4..(i + 4).saturating_add(n))
    {
        let (kind, fp) = ssh_key(blob);
        mark(d, "Kind", &kind, false);
        mark(d, "Fingerprint", &fp, false);
    }
    d.paragraph("Its numbers are not shown. Anyone who sees this file can use the key.");
}

fn ssh_public(d: &mut DocEvents, text: &[u8], known_hosts: bool) {
    let rows: Vec<Vec<String>> = ssh_lines(text)
        .take(MAX_NAMES)
        .map(|line| {
            let words: Vec<&str> = line.split_whitespace().collect();
            let at = words.iter().position(|w| is_ssh_type(w)).unwrap_or(0);
            let before = words[..at].join(" ");
            let blob = words
                .get(at + 1)
                .and_then(|b| data_encoding::BASE64.decode(b.as_bytes()).ok())
                .unwrap_or_default();
            let (kind, fp) = ssh_key(&blob);
            let comment = words.get(at + 2..).map(|c| c.join(" ")).unwrap_or_default();
            let mut options = before;
            if !known_hosts && options.contains("command=") {
                options.push_str("  ⚠ runs a fixed command");
            }
            if known_hosts && options.starts_with("|1|") {
                options = "(hashed host)".into();
            }
            vec![shown(&options), kind, fp, shown(&comment)]
        })
        .collect();
    let first = if known_hosts { "Host" } else { "Options" };
    d.heading(
        2,
        if known_hosts {
            "Known hosts"
        } else {
            "Public keys"
        },
    );
    d.table(
        &[first, "Kind", "Fingerprint", "Comment"].map(String::from),
        &rows,
    );
}

/// The report on the certificates and keys in `data`, laid out for `width`.
pub fn render(
    path: &Path,
    data: &[u8],
    time: Duration,
    budget: usize,
    width: usize,
) -> Result<Doc, String> {
    let mut d = DocEvents::new(MAX_EVENTS);
    let now = chrono::Utc::now().timestamp();
    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    let mut links: Vec<Link> = Vec::new();
    let mut certs = 0;
    let mut blocks = 0;
    let mut problems: Vec<String> = Vec::new();
    let head = &data[..data.len().min(MAX_SEARCH)];
    if pem_start(head).is_some() {
        for pem in Pem::iter_from_buffer(head) {
            if blocks >= MAX_BLOCKS || d.cut {
                problems.push(format!("only the first {MAX_BLOCKS} blocks are shown"));
                break;
            }
            let Ok(pem) = pem else {
                problems.push("a block that cannot be read".into());
                continue;
            };
            blocks += 1;
            if pem.contents.len() > MAX_BLOCK
                || !der_whole(&pem.contents) && pem.label != "OPENSSH PRIVATE KEY"
            {
                problems.push(format!(
                    "block {blocks} ({}) is too big or not valid DER",
                    shown(&pem.label)
                ));
                continue;
            }
            match pem.label.as_str() {
                "CERTIFICATE" | "TRUSTED CERTIFICATE" | "X509 CERTIFICATE" => {
                    certs += 1;
                    match certificate(&mut d, &pem.contents, certs, now) {
                        Ok(l) => links.push(l),
                        Err(e) => problems.push(e),
                    }
                }
                "CERTIFICATE REQUEST" | "NEW CERTIFICATE REQUEST" => request(&mut d, &pem.contents),
                "X509 CRL" => crl(&mut d, &pem.contents, now),
                "PRIVATE KEY"
                | "ENCRYPTED PRIVATE KEY"
                | "RSA PRIVATE KEY"
                | "EC PRIVATE KEY"
                | "DSA PRIVATE KEY" => {
                    private_key(&mut d, &pem.label, &pem.contents);
                }
                "OPENSSH PRIVATE KEY" => openssh_private(&mut d, &pem.contents),
                "PUBLIC KEY" => match SubjectPublicKeyInfo::from_der(&pem.contents) {
                    Ok((_, spki)) => {
                        d.heading(2, "Public key");
                        let (k, weak) = key_text(&spki);
                        mark(&mut d, "Kind", &k, weak);
                        mark(&mut d, "SHA-256", &fingerprint(&pem.contents), false);
                    }
                    Err(_) => problems.push("a public key that cannot be read".into()),
                },
                other => {
                    d.heading(2, &shown(other));
                    d.paragraph(&format!(
                        "A block of {} bytes, of a kind not read here.",
                        pem.contents.len()
                    ));
                }
            }
        }
    } else if der_whole(data) {
        if X509Certificate::from_der(data).is_ok() {
            certs = 1;
            if let Ok(l) = certificate(&mut d, data, 1, now) {
                links.push(l);
            }
        } else if X509CertificationRequest::from_der(data).is_ok() {
            request(&mut d, data);
        } else if CertificateRevocationList::from_der(data).is_ok() {
            crl(&mut d, data, now);
        } else if SubjectPublicKeyInfo::from_der(data).is_ok() {
            let (_, spki) = SubjectPublicKeyInfo::from_der(data).map_err(|e| e.to_string())?;
            d.heading(2, "Public key");
            let (k, weak) = key_text(&spki);
            mark(&mut d, "Kind", &k, weak);
        } else {
            private_key(&mut d, "PRIVATE KEY", data);
        }
    } else if ssh_lines(head).next().is_some() {
        ssh_public(&mut d, head, name.starts_with("known_hosts"));
    } else {
        return Err("No certificate or key this viewer can read".into());
    }
    let mut head_events = DocEvents::new(10_000);
    if certs > 1 {
        head_events.heading(1, &format!("{certs} certificates"));
        let ordered = links.windows(2).all(|w| {
            w[0].issuer == w[1].subject
                && match (&w[0].aki, &w[1].ski) {
                    (Some(a), Some(s)) => a == s,
                    _ => true,
                }
        });
        head_events.paragraph(if ordered {
            "In order: each is issued by the next, by name. No signature is verified here."
        } else {
            "Not a chain in order (or a bundle of separate certificates): checked by name only. No signature is verified here."
        });
    } else if certs == 1 {
        head_events.paragraph("No signature is verified here.");
    }
    for p in &problems {
        head_events.paragraph(&format!("⚠ {p}"));
    }
    let mut events = head_events.events;
    events.append(&mut d.events);
    let mut doc = markdown::from_events("", events, time, budget);
    if d.cut {
        doc.mark_cut();
    }
    doc.layout(width);
    Ok(doc)
}

fn request(d: &mut DocEvents, der: &[u8]) {
    d.heading(2, "Certificate request (CSR)");
    match X509CertificationRequest::from_der(der) {
        Ok((_, csr)) => {
            let info = &csr.certification_request_info;
            mark(d, "Subject", &shown(&info.subject.to_string()), false);
            let (k, weak) = key_text(&info.subject_pki);
            mark(d, "Public key", &k, weak);
            let sig = name_of(&csr.signature_algorithm.algorithm);
            mark(d, "Signature", &sig, weak_signature(&sig));
        }
        Err(_) => d.paragraph("It cannot be read."),
    }
}

fn crl(d: &mut DocEvents, der: &[u8], now: i64) {
    d.heading(2, "Revocation list (CRL)");
    match CertificateRevocationList::from_der(der) {
        Ok((_, list)) => {
            mark(d, "Issuer", &shown(&list.issuer().to_string()), false);
            mark(d, "Issued", &time_text(&list.last_update()), false);
            if let Some(next) = list.next_update() {
                mark(d, "Next update", &time_text(&next), next.timestamp() < now);
            }
            mark(
                d,
                "Revoked",
                &list.iter_revoked_certificates().count().to_string(),
                false,
            );
        }
        Err(_) => d.paragraph("It cannot be read."),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A self-signed certificate made for these tests (RSA 2048, SHA-256,
    /// valid from 2026 to 2036, CN=xylem.test, SAN xylem.test and 127.0.0.1).
    const CERT: &str = include_str!("../../tests/data/cert.pem");
    const KEY: &str = include_str!("../../tests/data/key.pem");

    fn report(name: &str, data: &[u8]) -> String {
        let doc = render(Path::new(name), data, Duration::from_secs(5), 1 << 22, 120).unwrap();
        (0..doc.line_count())
            .map(|i| doc.line(i).to_string())
            .collect::<Vec<_>>()
            .join("\n")
    }

    #[test]
    fn a_certificate_and_its_key() {
        assert!(detect(Path::new("c.pem"), CERT.as_bytes()));
        assert!(detect(Path::new("id_rsa"), KEY.as_bytes()));
        assert!(
            !detect(Path::new("tls.rs"), CERT.as_bytes()),
            "an example in source code"
        );
        assert!(!detect(Path::new("notes.md"), CERT.as_bytes()));
        let r = report("c.pem", CERT.as_bytes());
        for want in [
            "Certificate 1",
            "CN=xylem.test",
            "(self-signed)",
            "RSA 2048 bits",
            "sha256WithRSAEncryption",
            "DNS xylem.test",
            "IP 127.0.0.1",
            "SHA-256:",
            "No signature is verified here",
        ] {
            assert!(r.contains(want), "{want:?} in\n{r}");
        }
        let r = report("k.pem", KEY.as_bytes());
        assert!(r.contains("RSA 2048 bits (PKCS#8)"), "{r}");
        // The key's numbers are nowhere in the report.
        let body: String = KEY.lines().filter(|l| !l.starts_with("-----")).collect();
        let der = data_encoding::BASE64.decode(body.as_bytes()).unwrap();
        for chunk in der.chunks(16).skip(4).take(20) {
            let hex: String = chunk.iter().map(|b| format!("{b:02x}")).collect();
            assert!(!r.to_ascii_lowercase().contains(&hex));
        }
        assert!(!r.contains(&body[40..80]));
    }

    #[test]
    fn a_chain_is_checked_by_its_names() {
        let chain = include_str!("../../tests/data/chain.pem");
        let r = report("chain.pem", chain.as_bytes());
        assert!(r.contains("2 certificates"));
        assert!(r.contains("In order: each is issued by the next"), "{r}");
        assert!(
            r.contains("EC prime256v1") || r.contains("EC secp256r1"),
            "{r}"
        );
        assert!(r.contains("a CA"), "{r}");
        // The other way round, it is not a chain in order.
        let mut blocks: Vec<&str> = chain
            .split_inclusive("-----END CERTIFICATE-----\n")
            .collect();
        blocks.reverse();
        let r = report("rev.pem", blocks.concat().as_bytes());
        assert!(r.contains("Not a chain in order"), "{r}");
        let csr = include_str!("../../tests/data/leaf.csr");
        let r = report("leaf.csr", csr.as_bytes());
        assert!(
            r.contains("Certificate request") && r.contains("CN=leaf.xylem.test"),
            "{r}"
        );
    }

    #[test]
    fn ssh_keys() {
        let line = "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIOMqqnkVzrm0SdG6UOoqKLsabgH5C9okWi0dh2l9GKJl me@host\n";
        assert!(detect(Path::new("id.pub"), line.as_bytes()));
        let r = report("id.pub", line.as_bytes());
        assert!(
            r.contains("Ed25519 256 bits") && r.contains("SHA256:") && r.contains("me@host"),
            "{r}"
        );
        let auth = format!("command=\"/bin/backup\" {line}");
        let r = report("authorized_keys", auth.as_bytes());
        assert!(r.contains("runs a fixed command"), "{r}");
    }

    #[test]
    fn der_is_checked_before_any_parser() {
        assert!(!der_whole(b""));
        assert!(!der_whole(&[0x30, 0x80, 0, 0]), "indefinite length");
        assert!(!der_whole(&[0x30, 0x05, 1]), "past the end");
        assert!(!der_whole(&[0x30, 0x84, 0xff, 0xff, 0xff, 0xff]));
        assert!(der_whole(&[0x30, 0x03, 0x02, 0x01, 0x05]));
        let mut deep = Vec::new();
        for i in 0..=MAX_DER_DEPTH {
            let inner = (MAX_DER_DEPTH - i) * 2;
            deep.extend([0x30, inner as u8]);
        }
        assert!(!der_whole(&deep), "too deep");
        assert_eq!(bits(&[0x00, 0x80, 0]), 16);
        assert_eq!(bits(&[0x01, 0]), 9);
    }

    proptest::proptest! {
        #[test]
        fn hostile_blocks_never_panic(body in proptest::collection::vec(proptest::num::u8::ANY, 0..600), label in 0usize..8) {
            let labels = ["CERTIFICATE", "PRIVATE KEY", "RSA PRIVATE KEY", "EC PRIVATE KEY", "ENCRYPTED PRIVATE KEY", "OPENSSH PRIVATE KEY", "X509 CRL", "PUBLIC KEY"];
            let pem = format!(
                "-----BEGIN {0}-----\n{1}\n-----END {0}-----\n",
                labels[label],
                data_encoding::BASE64.encode(&body)
            );
            let _ = render(Path::new("x.pem"), pem.as_bytes(), Duration::from_secs(2), 1 << 20, 80);
            let _ = render(Path::new("x.der"), &body, Duration::from_secs(2), 1 << 20, 80);
            let _ = der_depth(&body);
        }

        #[test]
        fn a_real_certificate_with_bytes_changed_never_panics(changes in proptest::collection::vec((0usize..1200, proptest::num::u8::ANY), 1..20)) {
            let body: String = CERT.lines().filter(|l| !l.starts_with("-----")).collect();
            let mut der = data_encoding::BASE64.decode(body.as_bytes()).unwrap();
            for (at, b) in changes {
                if let Some(x) = der.get_mut(at) {
                    *x = b;
                }
            }
            let _ = render(Path::new("x.der"), &der, Duration::from_secs(2), 1 << 20, 80);
        }
    }
}
