# Third-Party Notices

This project depends on and draws from open-source work by others. This file
provides attribution as required by their licenses and as a courtesy to the
teams whose work makes this project possible.

---

## c2pa-rs — C2PA Rust SDK

**Project:** [c2pa-rs](https://github.com/contentauth/c2pa-rs)
**Copyright:** Copyright (c) 2020 Adobe. All rights reserved.
**License:** MIT OR Apache-2.0 (dual-licensed)
**Version used:** 0.78.6

The `c2pa` crate is the reference Rust implementation of the C2PA (Coalition for
Content Provenance and Authenticity) technical specification. It provides all
manifest reading, writing, signing, and validation functionality used by this
project.

### How We Use It

- **Runtime dependency:** `umrs-c2pa` depends on `c2pa = "0.78.6"` for all C2PA
  manifest operations (reading, signing, validation, trust checking).
- **Research reference:** Our trust list architecture, validation reporting, and
  improvement plan (see `docs/c2patool-research.md`) were informed by studying
  the c2pa-rs source code and documentation.
- **Test fixtures:** Integration tests may reference test images and certificate
  fixtures from the c2pa-rs source tree. These files remain in the c2pa-rs
  directory and are not redistributed in this repository.
- **Trust anchor files:** When deployed, operators may use trust anchor PEM
  bundles originating from the C2PA consortium's published trust infrastructure.

### License Text

The full license texts are available at:
- MIT: https://github.com/contentauth/c2pa-rs/blob/main/LICENSE-MIT
- Apache-2.0: https://github.com/contentauth/c2pa-rs/blob/main/LICENSE-APACHE

---

## OpenSSL

**Project:** [openssl](https://crates.io/crates/openssl) (Rust bindings)
**License:** Apache-2.0
**Usage:** ECDSA ephemeral certificate generation for C2PA signing.

---

## Other Dependencies

All other dependencies are listed in `Cargo.toml` with inline comments explaining
their purpose. Their licenses can be reviewed via `cargo license` or by inspecting
each crate on [crates.io](https://crates.io).
