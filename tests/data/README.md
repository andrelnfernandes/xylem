Files made only for the tests of the certificate viewer (src/viewer/certs.rs):

- cert.pem and key.pem: a self-signed certificate and its key. The key protects
  nothing.
- chain.pem: a certificate and the test CA that issued it; leaf.csr: its request.
  Their keys were not kept.
