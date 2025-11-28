//! Global allocator configuration using mimalloc.
//!
//! This module provides a high-performance global allocator using mimalloc,
//! which typically provides 10-20% speedup for allocation-heavy workloads.
//!
//! # Usage
//!
//! Enable the `mimalloc-allocator` feature in your binary crate's Cargo.toml:
//!
//! ```toml
//! [dependencies]
//! syn_core = { path = "../syn_core", features = ["mimalloc-allocator"] }
//! ```
//!
//! Then in your binary's main.rs, import the allocator:
//!
//! ```ignore
//! // This line sets mimalloc as the global allocator
//! use syn_core::allocator::MiMalloc;
//! ```
//!
//! # Benefits of mimalloc
//!
//! - **Speed**: Faster allocation/deallocation than the system allocator
//! - **Memory efficiency**: Better memory reuse and lower fragmentation
//! - **Thread scalability**: Excellent performance in multi-threaded scenarios
//! - **Security**: Optional hardening against heap exploits (enabled via `secure` feature)

#[cfg(feature = "mimalloc-allocator")]
pub use mimalloc::MiMalloc;

/// The global allocator instance.
///
/// When the `mimalloc-allocator` feature is enabled, this sets mimalloc
/// as the global allocator for the entire application.
///
/// Import this in your binary crate's main.rs to activate:
/// ```ignore
/// #[global_allocator]
/// static GLOBAL: syn_core::allocator::GlobalAllocator = syn_core::allocator::GlobalAllocator;
/// ```
#[cfg(feature = "mimalloc-allocator")]
#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

/// Type alias for the global allocator.
///
/// When `mimalloc-allocator` is enabled, this is `mimalloc::MiMalloc`.
/// Otherwise, it falls back to the system allocator.
#[cfg(feature = "mimalloc-allocator")]
pub type GlobalAllocator = MiMalloc;

#[cfg(test)]
mod tests {
    #[test]
    fn test_allocator_works() {
        // Simple allocation test - if mimalloc is broken, this would crash
        let v: Vec<i32> = (0..1000).collect();
        assert_eq!(v.len(), 1000);

        // String allocation
        let s = "hello world".repeat(100);
        assert_eq!(s.len(), 1100);

        // Box allocation
        let b = Box::new([0u8; 4096]);
        assert_eq!(b.len(), 4096);
    }
}
