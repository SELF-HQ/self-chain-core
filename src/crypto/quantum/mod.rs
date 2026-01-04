/// Post-quantum cryptography implementations for SELF Chain
///
/// This module provides quantum-resistant cryptographic algorithms:
/// - Kyber for key encapsulation (NIST PQC Round 3 selection) using pqcrypto-kyber
/// - SPHINCS+ for digital signatures (NIST PQC Round 3 selection) using pqcrypto-sphincsplus
///
/// Both implementations use the standalone pqcrypto implementations rather than OQS.
/// These implementations provide proper key management, serialization, and cryptographic guarantees.
pub mod kyber;
pub mod sphincs;
