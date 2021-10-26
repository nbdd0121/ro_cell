#![no_std]

use core::cell::UnsafeCell;
use core::fmt;
use core::mem::MaybeUninit;
use core::ops::Deref;

/// A cell that is readonly.
///
/// It is expected to remain readonly for most time. Some use cases include set-once global
/// variables. Construction and mutation of `RoCell` are allowed in unsafe code, and the safety
/// must be ensured by the caller.
///
/// This type should only be used for extremely performance critical scenarios; you
/// would normal want to use `OnceCell` instead. Mutating `RoCell` in multi-threaded is
/// extremely dangerous, do not take it lightly.
pub struct RoCell<T>(UnsafeCell<MaybeUninit<T>>);

unsafe impl<T: Send> Send for RoCell<T> {}
unsafe impl<T: Sync> Sync for RoCell<T> {}

impl<T> Drop for RoCell<T> {
    #[inline]
    fn drop(&mut self) {
        unsafe { core::ptr::drop_in_place((*self.0.get()).as_mut_ptr()) };
    }
}

impl<T> RoCell<T> {
    /// Create a new `RoCell` that is initialized already.
    #[inline]
    pub const fn new(value: T) -> Self {
        RoCell(UnsafeCell::new(MaybeUninit::new(value)))
    }

    /// Create a new `RoCell` that is uninitialized.
    ///
    /// # Safety
    ///
    /// RoCell can be read in safe code. Therefore, we make its construction unsafe to
    /// permitting uninit value. If `T` needs drop, the caller must ensure that RoCell is
    /// initialised or forgotten before it is dropped.
    #[inline]
    pub const unsafe fn new_uninit() -> Self {
        RoCell(UnsafeCell::new(MaybeUninit::uninit()))
    }

    /// Initialize a `RoCell`.
    ///
    /// # Safety
    ///
    /// The caller must guarantee that there are no other reference to the
    /// content of this `RoCell` exists.
    ///
    /// No synchronisation is handled by RoCell.
    /// The caller must guarantee that no other threads are accessing this
    /// RoCell and other threads are properly synchronised after the call.
    #[inline]
    pub unsafe fn init(this: &Self, value: T) {
        core::ptr::write((*this.0.get()).as_mut_ptr(), value);
    }

    /// Replace a `RoCell` and return old content.
    ///
    /// # Safety
    ///
    /// The caller must guarantee that there are no other reference to the
    /// content of this `RoCell` exists.
    ///
    /// No synchronisation is handled by `RoCell`.
    /// The caller must guarantee that no other threads are accessing this
    /// RoCell and other threads are properly synchronised after the call.
    #[inline]
    pub unsafe fn replace(this: &Self, value: T) -> T {
        core::mem::replace(RoCell::as_mut(this), value)
    }

    /// Get a mutable reference to this `RoCell`.
    ///
    /// # Safety
    ///
    /// The caller must guarantee that there are no other reference to the
    /// content of this `RoCell` exists, until the returned mutable reference
    /// is dropped.
    ///
    /// No synchronisation is handled by `RoCell`.
    /// The caller must guarantee that no other threads are accessing this
    /// RoCell and other threads are properly synchronised after
    /// manipulating the mutable reference.
    #[inline]
    pub unsafe fn as_mut(this: &Self) -> &mut T {
        &mut *(*this.0.get()).as_mut_ptr()
    }
}

impl<T> Deref for RoCell<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &T {
        unsafe { &*(*self.0.get()).as_ptr() }
    }
}

impl<T: fmt::Debug> fmt::Debug for RoCell<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self.deref(), f)
    }
}
