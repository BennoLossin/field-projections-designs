#[test]
#[doc(hidden)]
fn move_out_desugared() {
    let mut bx = Box::new(BranchConfig {
        main: "main".to_string(),
        dev: "dev".to_string(),
    });
    let main = bx.main;
    print(main);
    bx.main = "trunk".to_string();

    drop(bx.dev);
}

#[test]
#[doc(hidden)]
#[cfg(false)]
fn move_out_own_desugared() {
    use design::own::own;
    let mut cfg = own!(BranchConfig {
        main: "main".to_string(),
        dev: "dev".to_string(),
    });
    let main = cfg.main;
    print(main);
    cfg.main = "trunk".to_string();

    drop(cfg.dev);
}

/// Moving in and out of [`Box<T>`].
///
/// ## Sugared Code
///
/// ```
/// let mut bx = Box::new(BranchConfig {
///     main: "main".to_string(),
///     dev: "dev".to_string(),
/// });
/// let main = bx.main;
/// print(main);
/// bx.main = "trunk".to_string();
///
/// drop(bx.dev);
/// ```
///
/// ## Desugared code
///
/// ```
/// ==== DESUGARED ====
/// ```
///
/// [`Box<T>`]: std::boxed::Box
#[cfg_attr(doc, design::utils::desugared)]
pub fn move_out() {
    let mut bx = Box::new(BranchConfig {
        main: "main".to_string(),
        dev: "dev".to_string(),
    });
    let main = unsafe {
        let hdl = LocalHandle::new(&raw mut bx);
        let hdl = DerefPlace::deref_place(hdl);
        let subplace = <field_of!(BranchConfig, main)>::default();
        let hdl = ProjectPlace::project_place(hdl, subplace);
        ReadPlace::read_place(hdl)
    };
    print(main);
    unsafe {
        let hdl = LocalHandle::new(&raw mut bx);
        let hdl = DerefPlace::deref_place(hdl);
        let subplace = <field_of!(BranchConfig, main)>::default();
        let hdl = ProjectPlace::project_place(hdl, subplace);
        WritePlace::write_place(hdl, "trunk".to_string())
    };

    drop(unsafe {
        let hdl = LocalHandle::new(&raw mut bx);
        let hdl = DerefPlace::deref_place(hdl);
        let subplace = <field_of!(BranchConfig, dev)>::default();
        let hdl = ProjectPlace::project_place(hdl, subplace);
        ReadPlace::read_place(hdl)
    });
    unsafe {
        let hdl = LocalHandle::new(&raw mut bx);
        let hdl = DerefPlace::deref_place(hdl);
        let subplace = <field_of!(BranchConfig, main)>::default();
        let hdl = ProjectPlace::project_place(hdl, subplace);
        DropPlace::drop_place(hdl)
    };
    unsafe { DropHusk::drop_husk(&raw mut bx) };
    forget(bx);
}

/// Moving in and out of [`Own<T>`].
///
/// ## Sugared Code
///
/// ```
/// let mut own = @Own BranchConfig {
///     main: "main".to_string(),
///     dev: "dev".to_string(),
/// };
/// let main = own.main;
/// print(main);
/// bx.main = "trunk".to_string();
///
/// drop(own.dev);
/// ```
///
/// ## Desugared code
///
/// ```
/// ==== DESUGARED ====
/// ```
///
/// [`Box<T>`]: std::boxed::Box
#[cfg(false)]
#[cfg_attr(doc, design::utils::desugared)]
pub fn move_out_own() {
    let mut own = {
        let v = BranchConfig {
            main: "main".to_string(),
            dev: "dev".to_string(),
        };
    };
    let main = unsafe {
        let hdl = LocalHandle::new(&raw mut bx);
        let hdl = DerefPlace::deref_place(hdl);
        let subplace = <field_of!(BranchConfig, main)>::default();
        let hdl = ProjectPlace::project_place(hdl, subplace);
        ReadPlace::read_place(hdl)
    };
    print(main);
    unsafe {
        let hdl = LocalHandle::new(&raw mut bx);
        let hdl = DerefPlace::deref_place(hdl);
        let subplace = <field_of!(BranchConfig, main)>::default();
        let hdl = ProjectPlace::project_place(hdl, subplace);
        WritePlace::write_place(hdl, "trunk".to_string())
    };

    drop(unsafe {
        let hdl = LocalHandle::new(&raw mut bx);
        let hdl = DerefPlace::deref_place(hdl);
        let subplace = <field_of!(BranchConfig, dev)>::default();
        let hdl = ProjectPlace::project_place(hdl, subplace);
        ReadPlace::read_place(hdl)
    });
    unsafe {
        let hdl = LocalHandle::new(&raw mut bx);
        let hdl = DerefPlace::deref_place(hdl);
        let subplace = <field_of!(BranchConfig, main)>::default();
        let hdl = ProjectPlace::project_place(hdl, subplace);
        DropPlace::drop_place(hdl)
    };
    unsafe { DropHusk::drop_husk(&raw mut bx) };
    forget(bx);
}
