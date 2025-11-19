use std::any::Any;
use std::marker::PhantomData;
use std::mem::ManuallyDrop;
use std::ops::{Deref, DerefMut};
use std::panic::catch_unwind;
use std::ptr::NonNull;

pub struct Frame<F, R>
where
    F: Future + ?Sized,
    R: Resume<F> + ?Sized,
{
    ptr: NonNull<F>,
    _marker: PhantomData<R>,
}

impl<F, R> Frame<F, R>
where
    F: Future + ?Sized,
    R: Resume<F> + ?Sized,
{
    pub fn into_raw(frame: Frame<F, R>) -> *mut F {
        let ptr = frame.ptr.as_ptr();
        let _ = ManuallyDrop::new(frame);
        ptr
    }

    pub unsafe fn from_raw(ptr: *mut F) -> Frame<F, R> {
        Frame {
            ptr: unsafe { NonNull::new_unchecked(ptr) },
            _marker: PhantomData,
        }
    }
}

impl<F, R> Deref for Frame<F, R>
where
    F: Future + ?Sized,
    R: Resume<F> + ?Sized,
{
    type Target = F;

    fn deref(&self) -> &Self::Target {
        unsafe { self.ptr.as_ref() }
    }
}

impl<F, R> DerefMut for Frame<F, R>
where
    F: Future + ?Sized,
    R: Resume<F> + ?Sized,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.ptr.as_mut() }
    }
}

impl<F, R> Drop for Frame<F, R>
where
    F: Future + ?Sized,
    R: Resume<F> + ?Sized,
{
    fn drop(&mut self) {
        let result = catch_unwind(|| panic!("Frame was dropped")).unwrap_err();

        let frame = unsafe { Frame::from_raw(self.ptr.as_ptr()) };
        R::cancel(frame, result)
    }
}

pub trait Resume<F>
where
    F: Future + ?Sized,
{
    fn resume(frame: Frame<F, Self>, value: F::Output);
    fn cancel(frame: Frame<F, Self>, payload: Box<dyn Any + Send>);
}

pub trait Future {
    type Output;

    fn schedule<R>(frame: Frame<Self, R>)
    where
        R: Resume<Self>;
}
