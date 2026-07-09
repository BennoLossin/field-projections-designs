use core::io;
use std::{marker::PhantomData, os::fd::RawFd, ptr::Pointee};

use _02_handles::design::{ops::ProjectPlace, subplace::Subplace};

pub struct FileSlice<T> {
    fd: RawFd,
    value: PhantomData<T>,
}

pub struct FileSliceHandle<T> {
    fd: RawFd,
    value: PhantomData<T>,
}

impl<S> ProjectPlace<S> for FileSliceHandle<S::Source>
where
    S: Subplace,
    S::Source: Pointee<Metadata = ()>,
{
    type Projected = FileSliceHandle<S::Target>;

    unsafe fn project_place(self, subplace: S) -> Self::Projected {
        let FileSliceHandle { fd, value } = self;
        // TODO: add `offset` to pos of the fd in the kernel
        FileSliceHandle {
            fd,
            value: PhantomData,
        }
    }
}

impl<T> FileSlice<T> {
    pub fn read(&self) -> io::Result<T>
    where
        T: FromBytes,
    {
        todo!()
    }
}

pub trait FromBytes {}
