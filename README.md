# BIP-353: DNS Payment Instructions

## Overview
This repository is a Rust implementation of BIP-353 for DNS-based Bitcoin payment instructions. The proof of concept (POC) encodes Bitcoin payment instructions in DNSSEC-signed DNS TXT records, allowing for human-readable Bitcoin addresses. For example, a human-readable address like `₿alice@example.com` securely resolves to any type of Bitcoin payment instruction: on-chain addresses, Lightning invoices, or Lightning offers.

### Goals
The goal of this project is to make Bitcoin payments significantly more user-friendly while maintaining strong security guarantees through DNSSEC and avoiding privacy pitfalls of other approaches.

## Design Decisions
The implementation provides:

1. **DNS Resolution**: Resolves `₿user@domain` addresses to payment instructions.
2. **DNSSEC Validation**: Enforces security requirements from BIP-353 using `trust-dns-resolver` with validation enabled.
3. **Multiple Payment Types**: Supports on-chain addresses, Lightning invoices, and Lightning offers.
4. **Minimal FFI Surface Area**: Simple integration with both Bitcoin Core and HWI.
5. **Async Resolution**: Tokio-based asynchronous DNS resolution.

## Building and Installation

```bash
# Clone the repository
git clone https://github.com/frankomosh/bip353
cd bip353

# Build the library
cargo build --release

# Build with C API
cargo build --release --features ffi

# Build with Python bindings
cargo build --release --features python
```

## Test Architecture
The test suite is organized into three main components:

1. **Address Parsing Tests**: Tests for correctly parsing human-readable Bitcoin addresses.
2. **URI Parsing Tests**: Tests for parsing different types of Bitcoin payment URIs.

### Running Tests

```bash
# Run unit tests
cargo test

# Run integration tests (some require DNS setup)
cargo test -- --ignored
```

## Integration Points

### Bitcoin Core Integration (C API)

```c
// Create a resolver
BIP353_ResolverHandle* resolver = bip353_resolver_create();

// Resolve a DNS name
char* error_msg = NULL;
char* uri = NULL;
char* type = NULL;
bool is_reusable = false;
bool success = bip353_resolve(resolver, "user", "domain.com", &uri, &type, &is_reusable, &error_msg);

// Free resources
bip353_string_free(uri);
bip353_string_free(type);
bip353_string_free(error_msg);
bip353_resolver_free(resolver);
```

### Example RPC Implementation

```cpp
static UniValue resolvebitcoinaddress(const JSONRPCRequest& request)
{
    RPCHelpMan{"resolvebitcoinaddress",
        "Resolves a human-readable Bitcoin address (₿user@domain).",
        {
            {"address", RPCArg::Type::STR, RPCArg::Optional::NO, "The human-readable Bitcoin address"}
        },
        RPCResult{
            RPCResult::Type::OBJ, "", "",
            {
                {RPCResult::Type::STR, "uri", "The BIP-21 URI"},
                {RPCResult::Type::STR, "type", "The payment type"},
                {RPCResult::Type::BOOL, "is_reusable", "Whether the address is reusable"}
            }
        },
        RPCExamples{
            HelpExampleCli("resolvebitcoinaddress", "\"₿alice@example.com\"")
        },
    }.Check(request);

    std::string address = request.params[0].get_str();
    std::string user, domain;

    // Parse the address
    if (!BIP353ParseAddress(address, user, domain)) {
        throw JSONRPCError(RPC_INVALID_PARAMETER, "Invalid address format");
    }

    // Create resolver and resolve
    BIP353_ResolverHandle* resolver = bip353_resolver_create();
    char* uri = NULL;
    char* type = NULL;
    bool is_reusable = false;
    char* error_msg = NULL;

    bool success = bip353_resolve(resolver, user.c_str(), domain.c_str(), &uri, &type, &is_reusable, &error_msg);

    // Free the resolver
    bip353_resolver_free(resolver);

    if (!success) {
        std::string error = error_msg ? error_msg : "Unknown error";
        bip353_string_free(error_msg);
        throw JSONRPCError(RPC_INTERNAL_ERROR, error);
    }

    // Format result
    UniValue result(UniValue::VOBJ);
    result.pushKV("uri", std::string(uri));
    result.pushKV("type", std::string(type));
    result.pushKV("is_reusable", is_reusable);

    // Free strings
    bip353_string_free(uri);
    bip353_string_free(type);

    return result;
}
```

### HWI Integration

#### Python Bindings

```python
from bip353 import PyResolver, PyPaymentInstruction

# Create a resolver
resolver = PyResolver()

# Resolve a human-readable Bitcoin address
try:
    instruction = resolver.resolve_address("₿alice@example.com")
    print(f"URI: {instruction.uri}")
    print(f"Type: {instruction.payment_type}")
    print(f"Reusable: {instruction.is_reusable}")
    print(f"Parameters: {instruction.parameters}")
except Exception as e:
    print(f"Error: {e}")
```

## Developer Usage

### Using the Rust Library

```rust
use bip353::{Resolver, PaymentInstruction, PaymentType};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a resolver
    let resolver = Resolver::new()?;

    // Parse an address into user and domain
    let (user, domain) = Resolver::parse_address("₿alice@example.com")?;

    // Resolve the address
    let instruction = resolver.resolve(&user, &domain).await?;

    // Use the payment instruction
    println!("URI: {}", instruction.uri);

    // Handle different payment types
    match instruction.payment_type {
        PaymentType::OnChain => {},
        PaymentType::Lightning => {},
        PaymentType::LightningOffer => {},
        PaymentType::Unknown => {},
    }

    Ok(())
}
```

### Using the Python API

```python
from bip353 import PyResolver

def resolve_bitcoin_address(address):
    resolver = PyResolver()
    try:
        instruction = resolver.resolve_address(address)
        return {
            "uri": instruction.uri,
            "type": instruction.payment_type,
            "is_reusable": instruction.is_reusable,
            "parameters": instruction.parameters
        }
    except Exception as e:
        return {"error": str(e)}

# Example usage
print(resolve_bitcoin_address("₿alice@example.com"))
```

## Installation

```bash
# Install library
cargo install --path .

# Install Python bindings
pip install -e .
```

## Configuration Options

```rust
let mut config = ResolverConfig::default();
let mut opts = ResolverOpts::default();
opts.timeout_ms = 2000; // 2 seconds
let resolver = Resolver::with_opts(config, opts)?;
```

