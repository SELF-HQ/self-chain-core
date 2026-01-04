use crate::crypto::quantum::sphincs_pq::{SphincsKeys, SphincsVariant};
use crate::crypto::common::traits::{KeyPair, Signer, Verifier};

/// Test basic signing and verification for each SPHINCS+ variant
#[test]
fn test_all_sphincs_variants_sign_verify() {
    let variants = [
        SphincsVariant::Sha2128SSimple,
        SphincsVariant::Sha2128FSimple,
        SphincsVariant::Sha2256SSimple,
        SphincsVariant::Sha2256FSimple,
        SphincsVariant::Shake128SSimple,
        SphincsVariant::Shake128FSimple,
    ];

    for variant in variants.iter() {
        println!("Testing variant: {}", variant);
        
        // Generate a key pair for this variant
        let keys = SphincsKeys::new_with_variant(*variant)
            .expect(&format!("Failed to generate keys for variant {}", variant));
        
        // Test message
        let message = b"Testing SPHINCS+ variant signatures";
        
        // Sign the message
        let signature = keys.sign(message)
            .expect(&format!("Failed to sign message with variant {}", variant));
        
        // Verify with the same key should succeed
        let verify_result = keys.verify(message, &signature)
            .expect(&format!("Verification failed for variant {}", variant));
        
        assert!(verify_result, "Signature verification should succeed for variant {}", variant);
    }
}

/// Test the algorithm_id method for KeyPair trait implementation
#[test]
fn test_algorithm_id() {
    let variants = [
        (SphincsVariant::Sha2128SSimple, 0x01),
        (SphincsVariant::Sha2128FSimple, 0x02),
        (SphincsVariant::Sha2256SSimple, 0x03),
        (SphincsVariant::Sha2256FSimple, 0x04),
        (SphincsVariant::Shake128SSimple, 0x05),
        (SphincsVariant::Shake128FSimple, 0x06),
    ];

    for (variant, expected_id) in variants.iter() {
        // Generate key pair
        let keys = SphincsKeys::new_with_variant(*variant)
            .expect(&format!("Failed to generate keys for variant {}", variant));
        
        // Check both variant's algorithm_id and KeyPair trait's algorithm_id
        assert_eq!(variant.algorithm_id(), *expected_id);
        assert_eq!(keys.algorithm_id(), *expected_id);
    }
}

/// Test serialization and deserialization
#[test]
fn test_sphincs_serialization() {
    for variant in [
        SphincsVariant::Sha2128SSimple,
        SphincsVariant::Sha2256FSimple, // Testing a different variant
    ].iter() {
        // Generate a key pair
        let keys = SphincsKeys::new_with_variant(*variant)
            .expect(&format!("Failed to generate keys for variant {}", variant));
        
        // Test message
        let message = b"Message for serialization test";
        
        // Sign before serialization
        let signature1 = keys.sign(message)
            .expect("Failed to sign message before serialization");
        
        // Serialize the keys
        let serialized = keys.to_bytes().expect("Failed to serialize keys");
        
        // Deserialize the keys
        let deserialized = SphincsKeys::from_bytes(&serialized).expect("Failed to deserialize keys");
        
        // Check that the variant was preserved
        assert_eq!(deserialized.algorithm_id(), variant.algorithm_id());
        
        // Verify the original signature with the deserialized key
        let verify_result = deserialized.verify(message, &signature1)
            .expect("Verification with deserialized key failed unexpectedly");
        
        assert!(verify_result, "Signature verification with deserialized key should succeed");
        
        // If we have a secret key, sign with the deserialized key
        if let Some(_) = deserialized.private_key() {
            let signature2 = deserialized.sign(message)
                .expect("Failed to sign with deserialized key");
                
            // Verify the new signature with the original key
            let verify_cross = keys.verify(message, &signature2)
                .expect("Cross-verification failed unexpectedly");
                
            assert!(verify_cross, "Cross-verification should succeed");
        }
    }
}

/// Test that signatures don't verify with the wrong message
#[test]
fn test_sphincs_different_messages() {
    let keys = SphincsKeys::new_with_variant(SphincsVariant::Sha2128FSimple)
        .expect("Failed to generate SPHINCS+ keys");
    
    // Different test messages
    let message1 = b"Original test message";
    let message2 = b"Different test message";
    
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

/// Test that signatures don't verify with the wrong key
#[test]
fn test_sphincs_key_isolation() {
    // Generate two different key pairs
    let keys1 = SphincsKeys::new_with_variant(SphincsVariant::Sha2128FSimple)
        .expect("Failed to generate first SPHINCS+ keys");
    
    let keys2 = SphincsKeys::new_with_variant(SphincsVariant::Sha2128FSimple)
        .expect("Failed to generate second SPHINCS+ keys");
    
    // Test message
    let message = b"Test message for key isolation";
    
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

/// Test the public_key_only functionality
#[test]
fn test_public_key_only() {
    let keys = SphincsKeys::new_with_variant(SphincsVariant::Sha2256SSimple)
        .expect("Failed to generate SPHINCS+ keys");
    
    // Create a public-key-only copy
    let pub_only = keys.public_key_only();
    
    // Ensure the public key copy has no private key
    assert!(pub_only.private_key().is_none(), "Public-key-only copy should not have a private key");
    
    // Test message
    let message = b"Test for public key only verification";
    
    // Sign with the original keys
    let signature = keys.sign(message)
        .expect("Failed to sign message");
    
    // Verify with the public-key-only copy
    let verify_result = pub_only.verify(message, &signature)
        .expect("Verification with public key only failed unexpectedly");
    
    assert!(verify_result, "Signature verification should succeed with public key only");
    
    // Attempt to sign with public-key-only copy should fail
    let sign_result = pub_only.sign(message);
    assert!(sign_result.is_err(), "Signing with public key only should fail");
}

/// Test cross-variant incompatibility (signatures from one variant should not verify with another)
#[test]
fn test_cross_variant_incompatibility() {
    let variant1 = SphincsVariant::Sha2128FSimple;
    let variant2 = SphincsVariant::Sha2256FSimple;
    
    let keys1 = SphincsKeys::new_with_variant(variant1)
        .expect("Failed to generate keys with first variant");
    
    let keys2 = SphincsKeys::new_with_variant(variant2)
        .expect("Failed to generate keys with second variant");
    
    // Test message
    let message = b"Cross-variant testing message";
    
    // Sign with the first key
    let signature1 = keys1.sign(message)
        .expect("Failed to sign message with first variant");
    
    // Attempt to verify with the second key
    // This should fail, but not error out - we expect a false result
    let verify_result = keys2.verify(message, &signature1)
        .expect("Verification operation should not fail");
    
    assert!(!verify_result, "Signature verification should fail across different variants");
}
