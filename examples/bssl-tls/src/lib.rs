#![feature(arbitrary_self_types)]

use std::{
    marker::PhantomData,
    ops::Receiver,
};

use design::{
    lang_limits::adt_reflect,
    ops::place::{
        BorrowPlace,
        CreateHandle,
        DerefPlace,
        PlaceHandle,
        PlaceProxy,
        ProjectPlace,
        borrowck::{
            AccessKind,
            Lifetime,
        },
    },
    place::{
        LocalHandle,
        MutHandle,
        RefHandle,
        Subplace,
    },
};

/// TLS states are configurables.
pub trait Configurable<State> {
    type Config: ?Sized;
}

/// Initial state of a client.
pub struct ClientInit<App>(PhantomData<fn() -> App>);

/// Earliest point of time in a server to do last minute configurations.
pub struct ClientHello<App: ?Sized>(PhantomData<fn() -> App>);

pub enum TlsMode {}

impl<App> Configurable<ClientInit<App>> for TlsMode {
    type Config = ConfigForClientInit;
}

impl<App> Configurable<ClientHello<App>> for TlsMode {
    type Config = ConfigForClientHello<App>;
}

type Config<Mode, State> = <Mode as Configurable<State>>::Config;

pub struct TlsConnection<Mode: ?Sized, State: ?Sized>(
    PhantomData<(fn() -> Mode, fn() -> State)>,
);

#[allow(unused)]
pub struct TlsConnectionHandle<'a, Mode, State>(
    MutHandle<'a, TlsConnection<Mode, State>>,
);

impl<Mode: Configurable<State>, State> PlaceProxy
    for TlsConnection<Mode, State>
{
    type Target = Config<Mode, State>;
}

unsafe impl<'a, Mode: 'a + Configurable<State>, State: 'a>
    CreateHandle<Lifetime<'a>> for TlsConnection<Mode, State>
{
    type Handle = TlsConnectionHandle<'a, Mode, State>;
    const ACCESS: AccessKind = AccessKind::Exclusive;

    unsafe fn handle_from_raw(this: *const Self) -> Self::Handle {
        unsafe { TlsConnectionHandle(MutHandle::from_raw(this.cast_mut())) }
    }
}

impl<'a, Mode: Configurable<State>, State> PlaceHandle
    for TlsConnectionHandle<'a, Mode, State>
{
    type Target = Config<Mode, State>;
}

pub struct TlsCredentials;

impl TlsCredentials {
    async fn sign(&mut self) -> SignatureResult {
        todo!()
    }
    fn sign_with_evp(&mut self) -> Result<(), ()> {
        todo!()
    }
}

#[allow(unused)]
enum SignatureResult {
    PendingHsmResponse,
    Ok(Vec<u8>),
    KeyIsInMemory,
}

adt_reflect!(
    pub struct ConfigForClientInit {
        tls_config: TlsConfig,
    }

    pub struct ConfigForClientHello<App> {
        tls_config: TlsConfig,          // immutable
        client_hello: ClientHello<App>, // definitely immutable
        credentials: TlsCredentials,    // must be mutable
    }

    pub struct TlsConfig {}
);

pub trait TlsSubplaceInfo: Subplace {
    type Handle<'a>: PlaceHandle<Target = Self::Target>
    where
        Self: 'a;
}

impl<App> TlsSubplaceInfo for field_of!(ConfigForClientHello<App>, tls_config) {
    type Handle<'a>
        = RefHandle<'a, TlsConfig>
    where
        App: 'a;
}

impl<App> TlsSubplaceInfo
    for field_of!(ConfigForClientHello<App>, client_hello)
{
    type Handle<'a>
        = RefHandle<'a, ClientHello<App>>
    where
        App: 'a;
}

impl<App> TlsSubplaceInfo
    for field_of!(ConfigForClientHello<App>, credentials)
{
    type Handle<'a>
        = MutHandle<'a, TlsCredentials>
    where
        App: 'a;
}

impl TlsSubplaceInfo for field_of!(ConfigForClientInit, tls_config) {
    type Handle<'a> = MutHandle<'a, TlsConfig>;
}

unsafe impl<
    'a,
    Mode: Configurable<State>,
    State,
    S: 'a + TlsSubplaceInfo<Source = Config<Mode, State>>,
> ProjectPlace<S> for TlsConnectionHandle<'a, Mode, State>
{
    type Projected = S::Handle<'a>;

    unsafe fn project_place(self, _subplace: S) -> Self::Projected {
        todo!(
            "ideally we apply a pointer offset into the shared configuration space"
        )
    }
}

#[allow(async_fn_in_trait)]
pub trait TransitState {
    type Mode;
    type Next;
    async fn poll_until_next(
        self: TlsConnection<Self::Mode, Self>,
    ) -> Result<TlsConnection<Self::Mode, Self::Next>, ()>;
}

trait InitClient {
    async fn prepare_client_hello(config: &mut TlsConfig) -> Result<(), ()>;
}

pub struct ServerHello<App>(PhantomData<App>);
pub struct ClientOfferCredential<App>(PhantomData<App>);

impl<Mode, State: ?Sized> Receiver for TlsConnection<Mode, State> {
    type Target = State;
}

// This is our end goal:

impl<App: InitClient> TransitState for ClientInit<App> {
    type Mode = TlsMode;
    type Next = ServerHello<App>;

    /// ```
    /// App::prepare_client_hello(@self.tls_config).await?;
    ///
    /// TlsConnection::transit::<Self::Next>(self)
    /// ```
    async fn poll_until_next(
        mut self: TlsConnection<Self::Mode, Self>,
    ) -> Result<TlsConnection<Self::Mode, Self::Next>, ()> {
        // !!!!!!
        // App::prepare_client_hello(@self.tls_config).await?;
        // !!!!!!
        App::prepare_client_hello(unsafe {
            let hdl: LocalHandle<TlsConnection<TlsMode, ClientInit<App>>> =
                LocalHandle::new(&raw mut self);
            let hdl: TlsConnectionHandle<'_, TlsMode, ClientInit<App>> =
                DerefPlace::deref_place(hdl);
            let subplace =
                <field_of!(ConfigForClientInit, tls_config)>::default();
            let hdl: MutHandle<'_, TlsConfig> =
                ProjectPlace::project_place(hdl, subplace);
            BorrowPlace::borrow(hdl)
        })
        .await?;

        TlsConnection::transit::<Self::Next>(self)
    }
}

impl<Mode, State> TlsConnection<Mode, State> {
    fn transit<Next>(self) -> Result<TlsConnection<Mode, Next>, ()> {
        todo!()
    }
}

async fn yield_now() {}

trait HandleClientHello {
    async fn handle_client_hello(
        client_hello: &ClientHello<Self>,
        tls_config: &TlsConfig,
    ) -> Result<(), ()>;
}

impl<App: HandleClientHello> TransitState for ClientHello<App> {
    type Mode = TlsMode;
    type Next = ClientOfferCredential<App>;

    /// ```
    /// App::handle_client_hello(@self.client_hello, @self.tls_config).await?;
    /// loop {
    ///     match self.credentials.sign().await {
    ///         SignatureResult::PendingHsmResponse => yield_now().await,
    ///         SignatureResult::Ok(_) => break,
    ///         SignatureResult::KeyIsInMemory => {
    ///             self.credentials.sign_with_evp()?;
    ///         }
    ///     }
    /// }
    /// TlsConnection::transit::<Self::Next>(self)
    /// ```
    async fn poll_until_next(
        mut self: TlsConnection<Self::Mode, Self>,
    ) -> Result<TlsConnection<Self::Mode, Self::Next>, ()> {
        App::handle_client_hello(
            unsafe {
                let hdl = LocalHandle::new(&raw mut self);
                let hdl = DerefPlace::deref_place(hdl);
                let subplace = <field_of!(
                    ConfigForClientHello<App>,
                    client_hello
                )>::default();
                let hdl = ProjectPlace::project_place(hdl, subplace);
                BorrowPlace::<&ClientHello<App>>::borrow(hdl)
            },
            unsafe {
                let hdl = LocalHandle::new(&raw mut self);
                let hdl = DerefPlace::deref_place(hdl);
                let subplace =
                    <field_of!(ConfigForClientHello<App>, tls_config)>::default(
                    );
                let hdl = ProjectPlace::project_place(hdl, subplace);
                BorrowPlace::<&TlsConfig>::borrow(hdl)
            },
        )
        .await?;
        loop {
            match unsafe {
                let hdl = LocalHandle::new(&raw mut self);
                let hdl = DerefPlace::deref_place(hdl);
                let subplace = <field_of!(
                    ConfigForClientHello<App>,
                    credentials
                )>::default();
                let hdl = ProjectPlace::project_place(hdl, subplace);
                BorrowPlace::<&mut TlsCredentials>::borrow(hdl)
            }
            .sign()
            .await
            {
                SignatureResult::PendingHsmResponse => yield_now().await,
                SignatureResult::Ok(_) => break,
                SignatureResult::KeyIsInMemory => {
                    unsafe {
                        let hdl = LocalHandle::new(&raw mut self);
                        let hdl = DerefPlace::deref_place(hdl);
                        let subplace = <field_of!(
                            ConfigForClientHello<App>,
                            credentials
                        )>::default();
                        let hdl = ProjectPlace::project_place(hdl, subplace);
                        BorrowPlace::<&mut TlsCredentials>::borrow(hdl)
                    }
                    .sign_with_evp()?;
                }
            }
        }
        TlsConnection::transit::<Self::Next>(self)
    }
}
