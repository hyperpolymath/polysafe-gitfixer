# Security Policy

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 0.1.x   | :white_check_mark: |

## Reporting a Vulnerability

We take the security of polysafe-gitfixer seriously. If you believe you have found a security vulnerability, please report it responsibly.

### Where to Report

**Preferred**: Create a confidential issue at [GitLab Security Issue](https://gitlab.com/Hyperpolymath/polysafe-gitfixer/-/issues/new?issuable_template=security)

**Alternative**: Email the maintainer at the address listed in MAINTAINERS.md

### What to Include

- Type of vulnerability (e.g., path traversal, audit log tampering, capability bypass)
- Full paths of affected source files
- Step-by-step instructions to reproduce
- Proof-of-concept code if possible
- Impact assessment

### Response Timeline

- **Initial Response**: Within 72 hours
- **Status Update**: Within 7 days
- **Resolution Target**: Within 30 days (depending on severity and complexity)

### What to Expect

1. **Acknowledgment**: We will confirm receipt of your report
2. **Investigation**: We will investigate and determine the impact
3. **Fix Development**: A fix will be developed and tested
4. **Coordinated Disclosure**: We will coordinate disclosure timing with you
5. **Credit**: You will be credited in the security advisory (unless you prefer anonymity)

## Security Measures

This project implements the following security measures:

### Cryptographic Standards
- **SHA-256 only** for hash chains and integrity verification
- No MD5 or SHA1 for security purposes
- Ring library for cryptographic primitives

### Path Safety
- Capability-based path access control
- Path traversal prevention via canonicalization
- Symlink escape detection
- Subcapability permission restriction

### Audit Logging
- Append-only audit logs with hash chain integrity
- Tamper detection via chain verification
- fsync durability guarantees

### CI/CD Security
- CodeQL static analysis
- TruffleHog credential scanning
- OSSF Scorecard monitoring
- SHA-pinned GitHub Actions

## Security.txt

This repository follows RFC 9116. See `.well-known/security.txt` for machine-readable security contact information.

## Acknowledgments

We thank the following individuals for responsibly disclosing security issues:

_(None yet - be the first!)_
