use std::{pin::Pin, sync::Arc};

use _02_handles::{
    application::refs::RefHandle,
    design::ops::{BorrowPlace, DerefPlace, ProjectPlace, ReadPlace, WrapPlace},
};
use adt_reflect::adt_reflect;
use pin_project::pin_project;

use crate::{
    mutex::{InsideOfMutex, Mutex, MutexGuard},
    rcu::{self, Rcu, RcuGuard, RcuOld},
};

adt_reflect!(
    pub struct Driver {
        driver_data: Arc<Mutex<DriverData>>,
    }

    #[pin_project]
    pub struct DriverData {
        #[pin]
        shared: Shared,
    }

    #[pin_project]
    pub struct Shared {
        #[pin]
        data: Rcu<Box<Data>>,
    }

    pub struct Data {
        num: u32,
    }
);

impl Driver {
    /// Writing is easy and doesn't need field projections.
    pub fn write_data(&self, new_data: Box<Data>) {
        let mut guard: Pin<MutexGuard<'_, DriverData>> = self.driver_data.lock();
        let driver_data: Pin<&mut DriverData> = guard.as_mut();
        let data: Pin<&mut Rcu<Box<Data>>> = driver_data.project().shared.project().data;
        let old: RcuOld<Box<Data>> = data.write(new_data);
        drop(old); // runs `synchronize_rcu` & drops the old value
    }

    pub fn read_data(&self) -> u32 {
        Self::read_data_raw(&self.driver_data)
    }

    /// Reading needs field projections:
    /// ```
    /// let guard: RcuGuard = read_lock();
    /// let shared: &InsideOfMutex<Shared> = &data.shared;
    /// let data: &InsideOfMutex<Rcu<Box<Data>>> = &shared.data;
    /// let data: &Data = data.read(&guard);
    /// data.num
    /// ```
    /// Note that this can also be expressed succinctly:
    /// ```
    /// let guard: RcuGuard = read_lock();
    /// data.shared.data.read(&guard).num
    /// ```
    fn read_data_raw(data: &Mutex<DriverData>) -> u32 {
        let guard: RcuGuard = rcu::read_lock();

        let shared: &InsideOfMutex<Shared> = unsafe {
            let hdl_data: RefHandle<'_, Mutex<DriverData>> =
                DerefPlace::deref_place(&raw const data);

            let shared_subplace = <field_of!(DriverData, shared)>::default();
            let shared_subplace_wrapped = Mutex::wrap(shared_subplace);

            let hdl_shared: RefHandle<'_, InsideOfMutex<Shared>> =
                ProjectPlace::project_place(hdl_data, shared_subplace_wrapped);

            let shared: &InsideOfMutex<Shared> =
                BorrowPlace::<&InsideOfMutex<Shared>>::borrow(hdl_shared);
            shared
        };

        let data: &InsideOfMutex<Rcu<Box<Data>>> = unsafe {
            let hdl_shared: RefHandle<'_, InsideOfMutex<Shared>> =
                DerefPlace::deref_place(&raw const shared);

            let data_subplace = <field_of!(Shared, data)>::default();
            let data_subplace_wrapped = InsideOfMutex::wrap(data_subplace);

            let hdl_data: RefHandle<'_, InsideOfMutex<Rcu<Box<Data>>>> =
                ProjectPlace::project_place(hdl_shared, data_subplace_wrapped);

            let data: &InsideOfMutex<Rcu<Box<Data>>> =
                BorrowPlace::<&InsideOfMutex<Rcu<Box<Data>>>>::borrow(hdl_data);
            data
        };

        let data: &Data = InsideOfMutex::read(data, &guard);

        unsafe {
            let hdl_data: RefHandle<'_, Data> = DerefPlace::deref_place(&raw const data);
            let num_subplace = <field_of!(Data, num)>::default();
            let hdl_num: RefHandle<'_, u32> = ProjectPlace::project_place(hdl_data, num_subplace);
            ReadPlace::read_place(hdl_num)
        }
    }
}
