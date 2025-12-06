# Security Policy

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 0.1.x   | :white_check_mark: |

## Reporting a Vulnerability

We take security vulnerabilities seriously. If you discover a security issue,
please report it responsibly.

### How to Report

1. **Preferred Method**: Open a confidential security issue on GitLab
   - Go to Issues → New Issue → Check "This issue is confidential"

2. **Email**: Send details to the maintainers listed in MAINTAINERS.md
   - Include "SECURITY" in the subject line
   - Use PGP encryption if available

3. **Do NOT**:
   - Open public issues for security vulnerabilities
   - Disclose the vulnerability publicly before it's fixed
   - Exploit the vulnerability beyond proof-of-concept

### What to Include

- Description of the vulnerability
- Steps to reproduce
- Potential impact assessment
- Suggested fix (if any)
- Your contact information for follow-up

### Response Timeline

| Severity | Acknowledgment | Target Resolution |
|----------|----------------|-------------------|
| Critical | 4 hours        | 24 hours          |
| High     | 12 hours       | 72 hours          |
| Medium   | 24 hours       | 1 week            |
| Low      | 48 hours       | 1 month           |

### Severity Classification

- **Critical**: Remote code execution, data corruption, privilege escalation
- **High**: Path traversal, audit log tampering, authentication bypass
- **Medium**: Information disclosure, denial of service
- **Low**: Minor information leaks, documentation issues

## Security Design

polysafe-gitfixer is designed with security as a primary concern:

### Type-Safe Languages

All core components use memory-safe, type-safe languages:
- **Rust**: Memory safety without garbage collection
- **Haskell**: Strong static typing, totality checking
- **Elixir/OTP**: Process isolation, fault tolerance
- **Idris**: Dependent types for correctness proofs

### Capability-Based Security

The `capability` crate implements:
- **Path Traversal Prevention**: All filesystem access goes through `DirCapability`
- **Unforgeable Tokens**: Capabilities cannot be forged or escalated
- **Principle of Least Privilege**: Capabilities grant minimum required access

### Audit Logging

- **Hash-Chained Log**: Each entry includes SHA-256 hash of previous entry
- **Tamper Evidence**: Any modification breaks the chain
- **Append-Only**: Log entries cannot be modified or deleted

### Transactional Operations

- **RAII Cleanup**: Resources freed automatically on scope exit
- **Atomic Operations**: Write-to-temp, then rename
- **Rollback on Failure**: Incomplete operations are automatically reversed

## Security Practices

### Development

- All code requires review before merge
- Automated security scanning in CI/CD
- Dependency vulnerability monitoring
- SPDX license headers on all source files

### Testing

- Unit tests for security-critical functions
- Property-based testing for edge cases
- Integration tests for capability boundaries

### Dependencies

We minimize dependencies and prefer:
- Well-audited, widely-used libraries
- Libraries with security-focused maintainers
- Pure Rust implementations over C bindings where practical

## Acknowledgments

We maintain a security acknowledgments list for responsible disclosures.
Reporters may choose to be credited publicly or remain anonymous.

## Contact

For security questions that don't involve vulnerabilities, you may:
- Open a regular issue with the "security-question" label
- Ask in project discussions

---

This security policy follows [RFC 9116](https://www.rfc-editor.org/rfc/rfc9116) guidelines.
See `.well-known/security.txt` for machine-readable security contact information.
