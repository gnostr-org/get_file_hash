// Copyright (c) 2024 Yuki Kishimoto
// Distributed under the MIT software license

//! Atomic destructor

#![forbid(unsafe_code)]
#![warn(missing_docs)]

extern crate alloc;

use alloc::sync::Arc;
use core::fmt::{self, Debug};
use core::ops::{Deref, DerefMut};
use core::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

mod saturating;

use self::saturating::SaturatingUsize;

/// Atomic destroyer
pub trait AtomicDestroyer: Debug + Clone {
    /// Optional name to identify inner in logs/teminal
    #[cfg(feature = "tracing")]
    fn name(&self) -> Option<String> {
        None
    }

    /// Instructions to execute when all instances are dropped
    fn on_destroy(&self);
}

/// Atomic destructor
pub struct AtomicDestructor<T>
where
    T: AtomicDestroyer,
{
    destroyed: Arc<AtomicBool>,
    counter: Arc<AtomicUsize>,
    inner: T,
}

impl<T> Deref for AtomicDestructor<T>
where
    T: AtomicDestroyer,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> DerefMut for AtomicDestructor<T>
where
    T: AtomicDestroyer,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<T> fmt::Debug for AtomicDestructor<T>
where
    T: AtomicDestroyer,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AtomicDestructor")
            .field("destroyed", &self.destroyed)
            .field("counter", &self.counter)
            .field("inner", &self.inner)
            .finish()
    }
}

impl<T> Clone for AtomicDestructor<T>
where
    T: AtomicDestroyer,
{
    fn clone(&self) -> Self {
        // Increase counter
        let _value: usize = self.counter.saturating_increment(Ordering::SeqCst);

        #[cfg(feature = "tracing")]
        if let Some(name) = &self.inner.name() {
            tracing::debug!("{} cloned: atomic counter increased to {}", name, _value);
        }

        // Clone
        Self {
            destroyed: self.destroyed.clone(),
            counter: self.counter.clone(),
            inner: self.inner.clone(),
        }
    }
}

impl<T> Drop for AtomicDestructor<T>
where
    T: AtomicDestroyer,
{
    fn drop(&mut self) {
        if self.is_destroyed() {
            #[cfg(feature = "tracing")]
            if let Some(name) = &self.inner.name() {
                tracing::debug!("{} already destroyed.", name);
            }
        } else {
            // Decrease counter
            let value: usize = self.counter.saturating_decrement(Ordering::SeqCst);

            #[cfg(feature = "tracing")]
            if let Some(name) = &self.inner.name() {
                tracing::debug!("{} dropped: atomic counter decreased to {}", name, value);
            }

            // Check if it's time for destruction
            if value == 0 {
                #[cfg(feature = "tracing")]
                if let Some(name) = &self.inner.name() {
                    tracing::debug!("Destroying {} ...", name);
                }

                // Destroy
                self.inner.on_destroy();

                // Mark as destroyed
                self.destroyed.store(true, Ordering::SeqCst);

                #[cfg(feature = "tracing")]
                if let Some(name) = &self.inner.name() {
                    tracing::debug!("{} destroyed", name);
                }
            }
        }
    }
}

impl<T> AtomicDestructor<T>
where
    T: AtomicDestroyer,
{
    /// New wrapper
    pub fn new(inner: T) -> Self {
        Self {
            destroyed: Arc::new(AtomicBool::new(false)),
            counter: Arc::new(AtomicUsize::new(1)),
            inner,
        }
    }

    /// Check if destroyed
    pub fn is_destroyed(&self) -> bool {
        self.destroyed.load(Ordering::SeqCst)
    }
}
