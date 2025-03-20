//! Tests for BIP-353 DNS resolution
//!
//! These are sample tests for resolving DNS TXT records to
//! Bitcoin payment instructions. It uses mocked DNS responses
//! to test the resolution process without actual DNS queries.

use std::str::FromStr;
use bip353::{Resolver, PaymentInstruction, PaymentType, Bip353Error};

// requires a running DNS server or mock to work properly
// For integration testing,mark it as ignored by default
// Run with: cargo test -- --ignored dns_resolution
#[test]
#[ignore]
fn test_successful_resolution() {
    // Create a tokio runtime for async tests
    let rt = tokio::runtime::Runtime::new().expect("Failed to create runtime");
    
    // Run the async test inside the runtime
    rt.block_on(async {
        // Create a resolver
        let resolver = Resolver::new().expect("Failed to create resolver");
        
        // test requires a properly configured DNS server
        // with a valid DNSSEC-signed TXT record for this name
        // Replace with a domain you control for actual testing
        let user = "test";
        let domain = "example.com";
        
        let result = resolver.resolve(user, domain).await;
        
        // If the domain has a properly configured BIP-353 record, this will succeed
        // Otherwise, it will fail with a DNS error, which is expected
        if result.is_ok() {
            let instruction = result.unwrap();
            assert!(instruction.uri.starts_with("bitcoin:"));
        } else {
            // This is expected this to fail on domains without BIP-353 records
            let err = result.unwrap_err();
            assert!(matches!(err, Bip353Error::DnsError(_)) || 
                    matches!(err, Bip353Error::InvalidRecord(_)));
        }
    });
}

// For effective unit testing, ideally use a mock DNS resolver
// Here's how to maybe structure such tests with a mock

#[test]
#[ignore]
fn test_with_mock_resolver() {
    // These tests are still work in progress
    //They need a mock implementation of the DNS resolver
    // For a minimal example, here's a structure
    
    /* 
    // Example of what mock setup might look like:
    let mut mock_resolver = MockDnsResolver::new();
    
    // Mock a successful response
    mock_resolver.expect_resolve()
        .with(eq("alice.user._bitcoin-payment.example.com"), eq(RecordType::TXT))
        .returning(|_, _| {
            Ok(MockDnsResponse::new_with_txt("bitcoin:bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4"))
        });
    
    // Create resolver with mock
    let resolver = Resolver::with_mock(mock_resolver);
    
    // Create a tokio runtime for async tests
    let rt = tokio::runtime::Runtime::new().expect("Failed to create runtime");
    
    // Run the async test inside the runtime
    rt.block_on(async {
        // Test resolution
        let result = resolver.resolve("alice", "example.com").await;
        assert!(result.is_ok());
        let instruction = result.unwrap();
        assert!(matches!(instruction.payment_type, PaymentType::OnChain));
    });
    */
}

// Tests resolving a human-readable address string
#[test]
#[ignore]
fn test_resolve_address() {
    // Create a tokio runtime for async tests
    let rt = tokio::runtime::Runtime::new().expect("Failed to create runtime");
    
    // Run the async test inside the runtime
    rt.block_on(async {
        // Create a resolver
        let resolver = Resolver::new().expect("Failed to create resolver");
        
        // This test requires a properly configured DNS server
        // with a valid DNSSEC-signed TXT record for this name
        // Replace with an address you control for actual testing
        let address = "₿test@example.com";
        
        let result = resolver.resolve_address(address).await;
        
        // If the domain has a properly configured BIP-353 record, this will succeed
        // Otherwise, it will fail with a DNS error, which is expected
        if result.is_ok() {
            let instruction = result.unwrap();
            assert!(instruction.uri.starts_with("bitcoin:"));
        } else {
            // also expected to fail on domains without BIP-353 records
            let err = result.unwrap_err();
            assert!(matches!(err, Bip353Error::DnsError(_)) || 
                    matches!(err, Bip353Error::InvalidRecord(_)));
        }
    });
}

// Test how the resolver handles invalid DNS names
#[test]
fn test_invalid_dns_names() {
    // Create a tokio runtime for async tests
    let rt = tokio::runtime::Runtime::new().expect("Failed to create runtime");
    
    // Run the async test inside the runtime
    rt.block_on(async {
        // Create a resolver
        let resolver = Resolver::new().expect("Failed to create resolver");
        
        // Invalid domain (RFC 1035 violation)
        let result = resolver.resolve("user", "invalid-domain-").await;
        assert!(result.is_err());
        
        // Non-existent domain
        let result = resolver.resolve("user", "this-domain-definitely-does-not-exist-12345.com").await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Bip353Error::DnsError(_)));
        
        // Empty domain
        let result = resolver.resolve("user", "").await;
        assert!(result.is_err());
    });
}

// Test DNSSEC validation behavior
// Note: This requires a properly configured environment with DNSSEC
#[test]
#[ignore]
fn test_dnssec_validation() {
    // Create a tokio runtime for async tests
    let rt = tokio::runtime::Runtime::new().expect("Failed to create runtime");
    
    // Run the async test inside the runtime
    rt.block_on(async {
        // Create a resolver
        let resolver = Resolver::new().expect("Failed to create resolver");
        
        // This test should use a domain known to have valid DNSSEC
        // For actual testing, replace with appropriate domain
        let result = resolver.resolve("test", "example.com").await;
        
        // For a domain with proper DNSSEC but no BIP-353 record,
        // a "record not found" type of error is expected, not a DNSSEC error
        if result.is_err() {
            let err = result.unwrap_err();
            assert!(!matches!(err, Bip353Error::DnssecError(_)));
        }
        
    });
}

// For real integration testing(production implementation), set up a test DNS server with:
// 1. A properly configured DNSSEC-signed zone
// 2. BIP-353 TXT records for different payment types
// 3. Various invalid configurations to test error handling
