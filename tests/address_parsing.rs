//! Tests for BIP-353 human-readable address parsing
//!
//! contains tests for parsing and validating human-readable
//! Bitcoin addresses in the format user@domain or ₿user@domain.

use bip353::{Resolver, Bip353Error};

#[test]
fn test_valid_addresses() {
    // Regular user@domain format
    let result = Resolver::parse_address("alice@example.com");
    assert!(result.is_ok());
    let (user, domain) = result.unwrap();
    assert_eq!(user, "alice");
    assert_eq!(domain, "example.com");
    
    // With Bitcoin prefix
    let result = Resolver::parse_address("₿bob@bitcoin.org");
    assert!(result.is_ok());
    let (user, domain) = result.unwrap();
    assert_eq!(user, "bob");
    assert_eq!(domain, "bitcoin.org");
    
    // With whitespace
    let result = Resolver::parse_address("  charlie@example.org  ");
    assert!(result.is_ok());
    let (user, domain) = result.unwrap();
    assert_eq!(user, "charlie");
    assert_eq!(domain, "example.org");
    
    // With subdomain
    let result = Resolver::parse_address("dave@subdomain.example.com");
    assert!(result.is_ok());
    let (user, domain) = result.unwrap();
    assert_eq!(user, "dave");
    assert_eq!(domain, "subdomain.example.com");
    
    // With numbers and special chars in user part
    let result = Resolver::parse_address("user123_456@example.com");
    assert!(result.is_ok());
    let (user, domain) = result.unwrap();
    assert_eq!(user, "user123_456");
    assert_eq!(domain, "example.com");
    
    // With dash in domain
    let result = Resolver::parse_address("eve@example-domain.com");
    assert!(result.is_ok());
    let (user, domain) = result.unwrap();
    assert_eq!(user, "eve");
    assert_eq!(domain, "example-domain.com");
}

#[test]
fn test_invalid_addresses() {
    // Missing @
    let result = Resolver::parse_address("aliceexample.com");
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), Bip353Error::InvalidAddress(_)));
    
    // Empty user part
    let result = Resolver::parse_address("@example.com");
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), Bip353Error::InvalidAddress(_)));
    
    // Empty domain part
    let result = Resolver::parse_address("alice@");
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), Bip353Error::InvalidAddress(_)));
    
    // Multiple @ symbols
    let result = Resolver::parse_address("alice@example@com");
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), Bip353Error::InvalidAddress(_)));
    
    // Empty string
    let result = Resolver::parse_address("");
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), Bip353Error::InvalidAddress(_)));
    
    // Only whitespace
    let result = Resolver::parse_address("   ");
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), Bip353Error::InvalidAddress(_)));
}

#[test]
fn test_edge_cases() {
    // Multiple Bitcoin prefixes
    let result = Resolver::parse_address("₿₿alice@example.com");
    assert!(result.is_ok());
    let (user, domain) = result.unwrap();
    // The function should only strip one Bitcoin prefix
    assert_eq!(user, "₿alice");
    assert_eq!(domain, "example.com");
    
    // Bitcoin prefix in the middle
    let result = Resolver::parse_address("alice₿@example.com");
    assert!(result.is_ok());
    let (user, domain) = result.unwrap();
    assert_eq!(user, "alice₿");
    assert_eq!(domain, "example.com");
    
    // Very long user part
    let long_user = "a".repeat(64);
    let address = format!("{}@example.com", long_user);
    let result = Resolver::parse_address(&address);
    assert!(result.is_ok());
    let (user, domain) = result.unwrap();
    assert_eq!(user, long_user);
    assert_eq!(domain, "example.com");
    
    // Very long domain part
    let long_domain = format!("{}.com", "a".repeat(60));
    let address = format!("alice@{}", long_domain);
    let result = Resolver::parse_address(&address);
    assert!(result.is_ok());
    let (user, domain) = result.unwrap();
    assert_eq!(user, "alice");
    assert_eq!(domain, long_domain);
}

#[test]
fn test_idna_domains() {
    // Test with internationalized domain names would go here
    // For a minimal implementation, we're skipping actual IDNA conversion
    // but showing how we'd test it
    
    // Example with fake punycode
    let result = Resolver::parse_address("alice@xn--bcher-kva.example");
    assert!(result.is_ok());
    let (user, domain) = result.unwrap();
    assert_eq!(user, "alice");
    assert_eq!(domain, "xn--bcher-kva.example");
}