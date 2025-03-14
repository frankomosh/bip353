//! BIP-353 DNS Payment Instructions - Minimal implementation
//! 
//! This library provides the core functionality for resolving
//! human-readable Bitcoin addresses (₿user@domain) through DNS.

use std::error::Error;
use std::fmt;
use std::collections::HashMap;
use trust_dns_resolver::{TokioAsyncResolver, config::*};

/// Main error type for BIP-353 operations
#[derive(Debug)]
pub enum Bip353Error {
    DnsError(String),
    InvalidAddress(String),
    InvalidRecord(String),
    DnssecError(String),
}

impl fmt::Display for Bip353Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Bip353Error::DnsError(msg) => write!(f, "DNS error: {}", msg),
            Bip353Error::InvalidAddress(msg) => write!(f, "Invalid address: {}", msg),
            Bip353Error::InvalidRecord(msg) => write!(f, "Invalid record: {}", msg),
            Bip353Error::DnssecError(msg) => write!(f, "DNSSEC error: {}", msg),
        }
    }
}

impl Error for Bip353Error {}

impl From<trust_dns_resolver::error::ResolveError> for Bip353Error {
    fn from(err: trust_dns_resolver::error::ResolveError) -> Self {
        Bip353Error::DnsError(err.to_string())
    }
}

/// Payment instruction type
#[derive(Debug, Clone)]
pub enum PaymentType {
    OnChain,
    Lightning,
    LightningOffer,
    Unknown,
}

/// BIP-353 payment instruction
#[derive(Debug, Clone)]
pub struct PaymentInstruction {
    pub uri: String,
    pub payment_type: PaymentType,
    pub is_reusable: bool,
    pub parameters: HashMap<String, String>,
}

impl PaymentInstruction {
    /// Parse a payment instruction from a Bitcoin URI
    pub fn from_uri(uri: &str) -> Result<Self, Bip353Error> {
        if !uri.to_lowercase().starts_with("bitcoin:") {
            return Err(Bip353Error::InvalidRecord("URI must start with 'bitcoin:'".into()));
        }
        
        let mut parameters = HashMap::new();
        let mut payment_type = PaymentType::Unknown;
        let mut is_reusable = true;
        
        // Parse URI parameters
        if let Some(query_start) = uri.find('?') {
            let query = &uri[query_start+1..];
            for pair in query.split('&') {
                if let Some(eq_pos) = pair.find('=') {
                    let key = pair[..eq_pos].to_string();
                    let value = pair[eq_pos+1..].to_string();
                    parameters.insert(key, value);
                }
            }
        }
        
        // Determine payment type
        if parameters.contains_key("lightning") {
            payment_type = PaymentType::Lightning;
            is_reusable = false;
        } else if parameters.contains_key("lno") {
            payment_type = PaymentType::LightningOffer;
            is_reusable = true;
        } else if !uri[8..].contains('?') && uri.len() > 8 {
            // Simple on-chain address
            payment_type = PaymentType::OnChain;
            is_reusable = true;
        }
        
        Ok(PaymentInstruction {
            uri: uri.to_string(),
            payment_type,
            is_reusable,
            parameters,
        })
    }
}

/// BIP-353 resolver
pub struct Resolver {
    resolver: TokioAsyncResolver,
}

impl Resolver {
    /// Create a new resolver
    pub fn new() -> Result<Self, Bip353Error> {
        // Create a new resolver with DNSSEC validation
        let mut opts = ResolverOpts::default();
        opts.validate = true; // Enable DNSSEC validation
        
        let resolver = TokioAsyncResolver::tokio(
            ResolverConfig::default(),
            opts,
        )?;
        
        Ok(Self { resolver })
    }
    
    /// Parse a human-readable Bitcoin address
    pub fn parse_address(address: &str) -> Result<(String, String), Bip353Error> {
        let addr = address.trim();
        
        // Remove Bitcoin prefix if present
        let addr = addr.strip_prefix("₿").unwrap_or(addr);
        
        // Split by @
        let parts: Vec<&str> = addr.split('@').collect();
        if parts.len() != 2 {
            return Err(Bip353Error::InvalidAddress("Address must be in format user@domain".into()));
        }
        
        let user = parts[0].trim();
        let domain = parts[1].trim();
        
        if user.is_empty() || domain.is_empty() {
            return Err(Bip353Error::InvalidAddress("User and domain cannot be empty".into()));
        }
        
        Ok((user.to_string(), domain.to_string()))
    }
    
    /// Resolve a human-readable Bitcoin address
    pub async fn resolve(&self, user: &str, domain: &str) -> Result<PaymentInstruction, Bip353Error> {
        // Construct DNS name
        let dns_name = format!("{}.user._bitcoin-payment.{}", user, domain);
        
        // Query TXT records - with opts.validate=true, this will fail if DNSSEC validation fails
        let response = self.resolver.txt_lookup(&dns_name).await?;
        
        // Extract and concatenate TXT record strings
        let mut bitcoin_uris = Vec::new();
        
        for txt in response.iter() {
            let txt_data: Vec<String> = txt.txt_data()
                .iter()
                .map(|bytes| String::from_utf8_lossy(bytes).into_owned())
                .collect();
            
            let concatenated = txt_data.join("");
            
            if concatenated.to_lowercase().starts_with("bitcoin:") {
                bitcoin_uris.push(concatenated);
            }
        }
        
        // BIP-353 requires exactly one Bitcoin URI
        match bitcoin_uris.len() {
            0 => Err(Bip353Error::InvalidRecord("No Bitcoin URI found".into())),
            1 => PaymentInstruction::from_uri(&bitcoin_uris[0]),
            _ => Err(Bip353Error::InvalidRecord("Multiple Bitcoin URIs found".into())),
        }
    }
    
    /// Resolve a human-readable Bitcoin address string
    pub async fn resolve_address(&self, address: &str) -> Result<PaymentInstruction, Bip353Error> {
        let (user, domain) = Self::parse_address(address)?;
        self.resolve(&user, &domain).await
    }
}
