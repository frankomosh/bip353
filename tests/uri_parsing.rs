//! Tests for BIP-353 Bitcoin URI parsing
//!
//! Tests for parsing different types of Bitcoin URIs
//! into payment instructions, including on-chain addresses, Lightning 
//! invoices, and Lightning offers.

use bip353::{PaymentInstruction, PaymentType, Bip353Error};

#[test] 
fn test_onchain_addresses() {
    // Basic on-chain address
    let uri = "bitcoin:bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4";
    let result = PaymentInstruction::from_uri(uri);
    assert!(result.is_ok());
    let instruction = result.unwrap();
    assert!(matches!(instruction.payment_type, PaymentType::OnChain));
    assert!(instruction.is_reusable);
    assert_eq!(instruction.uri, uri);
    assert!(instruction.parameters.is_empty());
    
    // On-chain address with amount
    let uri = "bitcoin:bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4?amount=0.01";
    let result = PaymentInstruction::from_uri(uri);
    assert!(result.is_ok());
    let instruction = result.unwrap();
    assert!(matches!(instruction.payment_type, PaymentType::OnChain));
    assert!(instruction.is_reusable);
    assert_eq!(instruction.uri, uri);
    assert_eq!(instruction.parameters.get("amount"), Some(&"0.01".to_string()));
    
    // On-chain address with multiple parameters
    let uri = "bitcoin:bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4?amount=0.01&label=Test&message=Payment";
    let result = PaymentInstruction::from_uri(uri);
    assert!(result.is_ok());
    let instruction = result.unwrap();
    assert!(matches!(instruction.payment_type, PaymentType::OnChain));
    assert!(instruction.is_reusable);
    assert_eq!(instruction.uri, uri);
    assert_eq!(instruction.parameters.get("amount"), Some(&"0.01".to_string()));
    assert_eq!(instruction.parameters.get("label"), Some(&"Test".to_string()));
    assert_eq!(instruction.parameters.get("message"), Some(&"Payment".to_string()));
}

#[test]
fn test_lightning_invoices() {
    // Lightning invoice
    let invoice = "lnbc1pvjluezpp5qqqsyqcyq5rqwzqfqqqsyqcyq5rqwzqfqqqsyqcyq5rqwzqfqypqdpl2pkx2ctnv5sxxmmwwd5kgetjd9n5tsp5yprzpgf28qmkpq3lq";
    let uri = format!("bitcoin:?lightning={}", invoice);
    let result = PaymentInstruction::from_uri(&uri);
    assert!(result.is_ok());
    let instruction = result.unwrap();
    assert!(matches!(instruction.payment_type, PaymentType::Lightning));
    assert!(!instruction.is_reusable);
    assert_eq!(instruction.uri, uri);
    assert_eq!(instruction.parameters.get("lightning"), Some(&invoice.to_string()));
    
    // Lightning invoice with additional parameters
    let invoice = "lnbc1pvjluezpp5qqqsyqcyq5rqwzqfqqqsyqcyq5rqwzqfqqqsyqcyq5rqwzqfqypqdpl2pkx2ctnv5sxxmmwwd5kgetjd9n5tsp5yprzpgf28qmkpq3lq";
    let uri = format!("bitcoin:?lightning={}&label=Lightning%20Payment", invoice);
    let result = PaymentInstruction::from_uri(&uri);
    assert!(result.is_ok());
    let instruction = result.unwrap();
    assert!(matches!(instruction.payment_type, PaymentType::Lightning));
    assert!(!instruction.is_reusable);
    assert_eq!(instruction.uri, uri);
    assert_eq!(instruction.parameters.get("lightning"), Some(&invoice.to_string()));
    assert_eq!(instruction.parameters.get("label"), Some(&"Lightning%20Payment".to_string()));
}

#[test]
fn test_lightning_offers() {
    // Lightning offer
    let offer = "lno1pg257enxv4ezn9w8effvuw9h2f3upwuv9kzq8lqcc2cxk9gw29mkzmfxvtvz9j8c7dm4wa4zqnywept9xscrzve2qgrap0s4h6fe4m3pqnswk29uy087sx50tjj75s";
    let uri = format!("bitcoin:?lno={}", offer);
    let result = PaymentInstruction::from_uri(&uri);
    assert!(result.is_ok());
    let instruction = result.unwrap();
    assert!(matches!(instruction.payment_type, PaymentType::LightningOffer));
    assert!(instruction.is_reusable);
    assert_eq!(instruction.uri, uri);
    assert_eq!(instruction.parameters.get("lno"), Some(&offer.to_string()));
    
    // Lightning offer with additional parameters
    let offer = "lno1pg257enxv4ezn9w8effvuw9h2f3upwuv9kzq8lqcc2cxk9gw29mkzmfxvtvz9j8c7dm4wa4zqnywept9xscrzve2qgrap0s4h6fe4m3pqnswk29uy087sx50tjj75s";
    let uri = format!("bitcoin:?lno={}&label=Coffee", offer);
    let result = PaymentInstruction::from_uri(&uri);
    assert!(result.is_ok());
    let instruction = result.unwrap();
    assert!(matches!(instruction.payment_type, PaymentType::LightningOffer));
    assert!(instruction.is_reusable);
    assert_eq!(instruction.uri, uri);
    assert_eq!(instruction.parameters.get("lno"), Some(&offer.to_string()));
    assert_eq!(instruction.parameters.get("label"), Some(&"Coffee".to_string()));
}

#[test]
fn test_invalid_uris() {
    // Missing bitcoin: prefix
    let uri = "bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4";
    let result = PaymentInstruction::from_uri(uri);
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), Bip353Error::InvalidRecord(_)));
    
    // Wrong scheme
    let uri = "lightning:lnbc1pvjluezpp5qqqsyqcyq5rqwzqfqqqsyqcyq5rqwzqfqqqsyqcyq5rqwzqfqypqdpl2pkx2ctnv5sxxmmww";
    let result = PaymentInstruction::from_uri(uri);
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), Bip353Error::InvalidRecord(_)));
    
    // Empty URI
    let uri = "";
    let result = PaymentInstruction::from_uri(uri);
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), Bip353Error::InvalidRecord(_)));
    
    // Just the prefix
    let uri = "bitcoin:";
    let result = PaymentInstruction::from_uri(uri);
    assert!(result.is_ok()); // This is technically valid, just without a payment type
    let instruction = result.unwrap();
    assert!(matches!(instruction.payment_type, PaymentType::Unknown));
}

#[test]
fn test_complex_uris() {
    // URI with on-chain address and fallback lightning invoice
    let uri = "bitcoin:bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4?amount=0.01&lightning=lnbc1pvjluezpp5qqqsyqcyq5rqwzqfqqqsyqcyq5rqwzqfqqqsyqcyq5rqwzqfqypqdpl2pkx2ctnv5sxxmmww";
    let result = PaymentInstruction::from_uri(uri);
    assert!(result.is_ok());
    let instruction = result.unwrap();
    // Based on the simplified implementation, Lightning should take precedence
    assert!(matches!(instruction.payment_type, PaymentType::Lightning));
    assert!(!instruction.is_reusable);
    
    // URI with unusual parameter format
    let uri = "bitcoin:?lightning=lnbc1&param_without_value&empty_param=";
    let result = PaymentInstruction::from_uri(uri);
    assert!(result.is_ok());
    let instruction = result.unwrap();
    assert!(matches!(instruction.payment_type, PaymentType::Lightning));
    assert!(!instruction.is_reusable);
    // Only properly formed key=value pairs is included
    assert_eq!(instruction.parameters.get("lightning"), Some(&"lnbc1".to_string()));
    assert_eq!(instruction.parameters.get("empty_param"), Some(&"".to_string()));
    // param_without_value should be ignored in this simple implementation
}

#[test]
fn test_case_sensitivity() {
    // Test case insensitivity of the bitcoin: prefix
    let uri = "BiTcOiN:bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4";
    let result = PaymentInstruction::from_uri(uri);
    assert!(result.is_ok());
    let instruction = result.unwrap();
    assert!(matches!(instruction.payment_type, PaymentType::OnChain));
    
    // Case sensitivity for parameter keys (impl should preserve case)
    let uri = "bitcoin:?LiGhTnInG=lnbc1&AmOuNt=0.01";
    let result = PaymentInstruction::from_uri(uri);
    assert!(result.is_ok());
    let instruction = result.unwrap();
    // impl looks for lowercase keys in payment_type determination
    assert!(matches!(instruction.payment_type, PaymentType::Unknown));
    assert_eq!(instruction.parameters.get("LiGhTnInG"), Some(&"lnbc1".to_string()));
    assert_eq!(instruction.parameters.get("AmOuNt"), Some(&"0.01".to_string()));
}