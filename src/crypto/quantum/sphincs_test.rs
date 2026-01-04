use crate::crypto::quantum::sphincs::{SphincsKeys, SphincsVariant};
use crate::crypto::common::traits::{KeyPair, Signer, Verifier};

#[test]
fn test_sphincs_sign_verify() {
    // Generate a key pair
    let keys = SphincsKeys::new_with_variant(SphincsVariant::Sha3256Small)
        .expect("Failed to generate SPHINCS+ keys");
    
    // Test message
    let message = b"Security validation test message";
    
    // Sign the message with the original keys
    let signature = keys.sign(message)
        .expect("Failed to sign message");
    
    // Verify with the same key should succeed
    let verify_result = keys.verify(message, &signature)
        .expect("Verification failed unexpectedly");
    
    assert!(verify_result, "Signature verification should succeed with correct key");
    
    // Create a public key only copy
    let public_only = keys.public_key_only();
    
    // Verify with public key only should also succeed
    let verify_pub_only = public_only.verify(message, &signature)
        .expect("Verification with public key only failed unexpectedly");
    
    assert!(verify_pub_only, "Signature verification should succeed with public key only");
    
    // Serialize and deserialize the keys
    let serialized = keys.to_bytes().expect("Failed to serialize keys");
    let deserialized = SphincsKeys::from_bytes(&serialized).expect("Failed to deserialize keys");
    
    // Verify the signature with deserialized keys
    let verify_deserialized = deserialized.verify(message, &signature)
        .expect("Verification with deserialized key failed unexpectedly");
    
    assert!(verify_deserialized, "Signature verification should succeed with deserialized key");
    
    // Create signature with the deserialized key (if it has a secret key)
    if let Some(_) = deserialized.private_key() {
        let sig2 = deserialized.sign(message)
            .expect("Failed to sign with deserialized key");
            
        // Verify the new signature with the original key
        let verify_cross = keys.verify(message, &sig2)
            .expect("Cross-verification failed unexpectedly");
            
        assert!(verify_cross, "Cross-verification should succeed");
    }
}

#[test]
fn test_sphincs_different_messages() {
    // Generate a key pair
    let keys = SphincsKeys::new_with_variant(SphincsVariant::Sha3256Small)
        .expect("Failed to generate SPHINCS+ keys");
    
    // Test messages
    let message1 = b"Security validation test message 1";
    let message2 = b"Security validation test message 2";
    
    // Sign the first message
    let signature = keys.sign(message1)
        .expect("Failed to sign message");
    
    // Verify with the correct message should succeed
    let verify_correct = keys.verify(message1, &signature)
        .expect("Verification failed unexpectedly");
    
    assert!(verify_correct, "Signature verification should succeed with correct message");
    
    // Verify with a different message should fail
    let verify_incorrect = keys.verify(message2, &signature)
        .expect("Verification operation failed unexpectedly");
    
    assert!(!verify_incorrect, "Signature verification should fail with incorrect message");
}

#[test]
fn test_sphincs_key_isolation() {
    // Generate two different key pairs
    let keys1 = SphincsKeys::new_with_variant(SphincsVariant::Sha3256Small)
        .expect("Failed to generate first SPHINCS+ keys");
    
    let keys2 = SphincsKeys::new_with_variant(SphincsVariant::Sha3256Small)
        .expect("Failed to generate second SPHINCS+ keys");
    
    // Test message
    let message = b"Security validation test message";
    
    // Sign with the first key
    let signature1 = keys1.sign(message)
        .expect("Failed to sign message with first key");
    
    // Verify with the first key should succeed
    let verify_key1 = keys1.verify(message, &signature1)
        .expect("Verification with first key failed unexpectedly");
    
    assert!(verify_key1, "Signature verification should succeed with correct key");
    
    // Verify with the second key should fail
    let verify_key2 = keys2.verify(message, &signature1)
        .expect("Verification operation with second key failed unexpectedly");
    
    assert!(!verify_key2, "Signature verification should fail with incorrect key");
}
