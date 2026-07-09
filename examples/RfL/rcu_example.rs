use std::sync::Arc;

use design::{
    application::{
        arc::ArcHandle,
        refs::{MutHandle, RefHandle},
    },
    design::{
        locals::LocalHandle,
        ops::{BorrowPlace, DerefHandle, DerefPlace, ProjectPlace, ReadPlace, WrapPlace},
    },
};
use adt_reflect::adt_reflect;

use crate::{
    mutex::{InsideOfMutex, Mutex, MutexGuard},
    overwrite::{Shield, ShieldHandle},
    rcu::{self, Rcu, RcuGuard, RcuOld},
};

adt_reflect!(
    pub struct Driver {
        driver_data: Arc<Mutex<DriverData>>,
    }

    pub struct DriverData {
        shared: Shared,
    }

    pub struct Shared {
        data: Rcu<Box<Data>>,
    }

    pub struct Data {
        num: u32,
    }
);

/// Workarounds for language/compiler limitations.
///
/// These would go away when more features are added to the compiler/language:
/// - negative reasoning for overlap checks of impls
mod lang_limits {
    use design::design::ops::PlaceHandle;

    use crate::overwrite::ShieldableSubplace;

    use super::*;

    unsafe impl ShieldableSubplace for field_of!(DriverData, shared) {
        type StructualShielding<H: PlaceHandle<Target = Self::Target>> = ShieldHandle<H>;

        unsafe fn from_shielded<H: PlaceHandle<Target = Self::Target>>(
            handle: H,
        ) -> Self::StructualShielding<H> {
            unsafe { ShieldHandle::new_unchecked(handle) }
        }
    }

    unsafe impl ShieldableSubplace for field_of!(Shared, data) {
        type StructualShielding<H: PlaceHandle<Target = Self::Target>> = ShieldHandle<H>;

        unsafe fn from_shielded<H: PlaceHandle<Target = Self::Target>>(
            handle: H,
        ) -> Self::StructualShielding<H> {
            unsafe { ShieldHandle::new_unchecked(handle) }
        }
    }
}

impl Driver {
    /// Writing needs field projections to handle the `Shield` abstraction nicely:
    /// ```
    /// let mut guard: Shield<MutexGuard<'_, DriverData>> = self.driver_data.lock();
    /// let data: Shield<&mut Rcu<Box<Data>>> = @guard.shared.data;
    /// let old: RcuOld<Box<Data>> = data.write(new_data);
    /// drop(old); // runs `synchronize_rcu` & drops the old value
    /// ```
    #[expect(unused_mut)]
    pub fn write_data(&self, new_data: Box<Data>) {
        let tmp: &Mutex<DriverData> = unsafe {
            let self_hdl: RefHandle<'_, Driver> = DerefHandle::handle_from_raw(&raw const self);
            let driver_data_subplace = <field_of!(Driver, driver_data)>::default();
            let driver_data_hdl: RefHandle<'_, Arc<Mutex<DriverData>>> =
                ProjectPlace::project_place(self_hdl, driver_data_subplace);
            let mutex_hdl: ArcHandle<Mutex<DriverData>> = DerefPlace::deref_place(driver_data_hdl);
            BorrowPlace::<&Mutex<DriverData>>::borrow(mutex_hdl)
        };

        let mut guard: Shield<MutexGuard<'_, DriverData>> = Mutex::lock(tmp);

        let data: Shield<&mut Rcu<Box<Data>>> = unsafe {
            let guard_hdl: LocalHandle<Shield<MutexGuard<'_, DriverData>>> =
                LocalHandle::new(&raw const guard);
            let driver_data_hdl: ShieldHandle<MutHandle<'_, DriverData>> =
                DerefPlace::deref_place(guard_hdl);
            let shared_subplace = <field_of!(DriverData, shared)>::default();
            let shared_hdl: ShieldHandle<MutHandle<'_, Shared>> =
                ProjectPlace::project_place(driver_data_hdl, shared_subplace);
            let data_subplace = <field_of!(Shared, data)>::default();
            let data_hdl: ShieldHandle<MutHandle<'_, Rcu<Box<Data>>>> =
                ProjectPlace::project_place(shared_hdl, data_subplace);
            BorrowPlace::<Shield<&mut Rcu<Box<Data>>>>::borrow(data_hdl)
        };

        let old: RcuOld<Box<Data>> = Rcu::write(data, new_data);
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
            let data_hdl: LocalHandle<&Mutex<DriverData>> = LocalHandle::new(&raw const data);
            let data_hdl: RefHandle<'_, Mutex<DriverData>> = DerefPlace::deref_place(data_hdl);

            let shared_subplace = <field_of!(DriverData, shared)>::default();
            let shared_subplace_wrapped = Mutex::wrap(shared_subplace);

            let shared_hdl: RefHandle<'_, InsideOfMutex<Shared>> =
                ProjectPlace::project_place(data_hdl, shared_subplace_wrapped);

            let shared: &InsideOfMutex<Shared> =
                BorrowPlace::<&InsideOfMutex<Shared>>::borrow(shared_hdl);
            shared
        };

        let data: &InsideOfMutex<Rcu<Box<Data>>> = unsafe {
            let shared_hdl: LocalHandle<&InsideOfMutex<Shared>> =
                LocalHandle::new(&raw const shared);
            let shared_hdl: RefHandle<'_, InsideOfMutex<Shared>> =
                DerefPlace::deref_place(shared_hdl);

            let data_subplace = <field_of!(Shared, data)>::default();
            let data_subplace_wrapped = InsideOfMutex::wrap(data_subplace);

            let data_hdl: RefHandle<'_, InsideOfMutex<Rcu<Box<Data>>>> =
                ProjectPlace::project_place(shared_hdl, data_subplace_wrapped);

            let data: &InsideOfMutex<Rcu<Box<Data>>> =
                BorrowPlace::<&InsideOfMutex<Rcu<Box<Data>>>>::borrow(data_hdl);
            data
        };

        let data: &Data = InsideOfMutex::read(data, &guard);

        unsafe {
            let data_hdl: RefHandle<'_, Data> = DerefHandle::handle_from_raw(&raw const data);
            let num_subplace = <field_of!(Data, num)>::default();
            let num_hdl: RefHandle<'_, u32> = ProjectPlace::project_place(data_hdl, num_subplace);
            ReadPlace::read_place(num_hdl)
        }
    }
}

#[test]
fn main() {
    let driver = Driver {
        driver_data: Arc::new(Mutex::new(DriverData {
            shared: Shared {
                data: Rcu::new(Box::new(Data { num: 42 })),
            },
        })),
    };
    let data = driver.read_data();
    assert_eq!(data, 42);
    driver.write_data(Box::new(Data { num: 70 }));
    let data = driver.read_data();
    assert_eq!(data, 70);
}
