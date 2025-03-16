//! Minimal Python bindings for BIP-353
//!
//! These bindings provide a simple Python API for HWI integration.

use pyo3::prelude::*;
use pyo3::exceptions::{PyRuntimeError, PyValueError};
use pyo3::wrap_pyfunction;
use tokio::runtime::Runtime;

use crate::{Bip353Error, PaymentInstruction, Resolver};

/// Convert a BIP-353 error to a Python exception
fn to_py_err(err: Bip353Error) -> PyErr {
    match err {
        Bip353Error::InvalidAddress(_) => PyValueError::new_err(err.to_string()),
        _ => PyRuntimeError::new_err(err.to_string()),
    }
}

/// Python wrapper for the resolver
#[pyclass]
struct PyResolver {
    resolver: Resolver,
    rt: Runtime,
}

#[pymethods]
impl PyResolver {
    /// Create a new resolver
    #[new]
    fn new() -> PyResult<Self> {
        let resolver = Resolver::new().map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
        let rt = Runtime::new().map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
        
        Ok(Self { resolver, rt })
    }
    
    /// Resolve a human-readable Bitcoin address
    fn resolve_address(&self, address: &str) -> PyResult<PyPaymentInstruction> {
        let instruction = self.rt.block_on(self.resolver.resolve_address(address))
            .map_err(to_py_err)?;
        
        Ok(PyPaymentInstruction { instruction })
    }
    
    /// Parse a human-readable Bitcoin address
    fn parse_address(&self, address: &str) -> PyResult<(String, String)> {
        Resolver::parse_address(address).map_err(to_py_err)
    }
}

/// Python wrapper for payment instructions
#[pyclass]
struct PyPaymentInstruction {
    instruction: PaymentInstruction,
}

#[pymethods]
impl PyPaymentInstruction {
    /// Get the URI
    #[getter]
    fn uri(&self) -> String {
        self.instruction.uri.clone()
    }
    
    /// Get the payment type
    #[getter]
    fn payment_type(&self) -> String {
        match self.instruction.payment_type {
            crate::PaymentType::OnChain => "on-chain".to_string(),
            crate::PaymentType::Lightning => "lightning".to_string(),
            crate::PaymentType::LightningOffer => "lightning-offer".to_string(),
            crate::PaymentType::Unknown => "unknown".to_string(),
        }
    }
    
    /// Is the payment instruction reusable?
    #[getter]
    fn is_reusable(&self) -> bool {
        self.instruction.is_reusable
    }
    
    /// Get parameters
    #[getter]
    fn parameters(&self, py: Python) -> PyObject {
        let dict = PyDict::new(py);
        
        for (key, value) in &self.instruction.parameters {
            dict.set_item(key, value).unwrap();
        }
        
        dict.into()
    }
}

/// Python module
#[pymodule]
fn bip353(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyResolver>()?;
    m.add_class::<PyPaymentInstruction>()?;
    
    Ok(())
}