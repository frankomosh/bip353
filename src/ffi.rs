//! Minimal FFI bindings for BIP-353
//!
//! These bindings would provide a simple C API for Bitcoin Core integration.

use std::ffi::{c_char, CStr, CString};
use std::ptr;
use tokio::runtime::Runtime;

use crate::{Bip353Error, PaymentInstruction, Resolver};

/// Opaque pointer for the resolver
pub struct ResolverPtr(*mut Resolver);

/// Create a new resolver
#[no_mangle]
pub extern "C" fn bip353_resolver_create() -> *mut ResolverPtr {
    match Resolver::new() {
        Ok(resolver) => {
            let resolver_ptr = Box::new(resolver);
            let ptr = Box::new(ResolverPtr(Box::into_raw(resolver_ptr)));
            Box::into_raw(ptr)
        }
        Err(_) => ptr::null_mut(),
    }
}

/// Free a resolver
#[no_mangle]
pub extern "C" fn bip353_resolver_free(ptr: *mut ResolverPtr) {
    if !ptr.is_null() {
        unsafe {
            let resolver_ptr = Box::from_raw(ptr);
            if !resolver_ptr.0.is_null() {
                let _ = Box::from_raw(resolver_ptr.0);
            }
        }
    }
}

/// Resolve a human-readable Bitcoin address
#[no_mangle]
pub extern "C" fn bip353_resolve(
    ptr: *mut ResolverPtr,
    address: *const c_char,
    uri_out: *mut *mut c_char,
    type_out: *mut *mut c_char,
    is_reusable_out: *mut bool,
) -> bool {
    if ptr.is_null() || address.is_null() || uri_out.is_null() || type_out.is_null() || is_reusable_out.is_null() {
        return false;
    }
    
    let resolver_ptr = unsafe { &*ptr };
    let resolver = unsafe { &*resolver_ptr.0 };
    
    let address_str = match unsafe { CStr::from_ptr(address) }.to_str() {
        Ok(s) => s,
        Err(_) => return false,
    };
    
    // Create a new runtime for async resolution
    let rt = match Runtime::new() {
        Ok(rt) => rt,
        Err(_) => return false,
    };
    
    // Resolve the address
    match rt.block_on(resolver.resolve_address(address_str)) {
        Ok(instruction) => {
            // Set the URI
            unsafe {
                match CString::new(instruction.uri) {
                    Ok(uri_cstring) => {
                        *uri_out = uri_cstring.into_raw();
                    }
                    Err(_) => return false,
                }
                
                // Set the type
                let type_str = match instruction.payment_type {
                    crate::PaymentType::OnChain => "on-chain",
                    crate::PaymentType::Lightning => "lightning",
                    crate::PaymentType::LightningOffer => "lightning-offer",
                    crate::PaymentType::Unknown => "unknown",
                };
                
                match CString::new(type_str) {
                    Ok(type_cstring) => {
                        *type_out = type_cstring.into_raw();
                    }
                    Err(_) => return false,
                }
                
                // Set is_reusable
                *is_reusable_out = instruction.is_reusable;
            }
            
            true
        }
        Err(_) => false,
    }
}

/// Free a string returned by the library
#[no_mangle]
pub extern "C" fn bip353_string_free(ptr: *mut c_char) {
    if !ptr.is_null() {
        unsafe {
            let _ = CString::from_raw(ptr);
        }
    }
}