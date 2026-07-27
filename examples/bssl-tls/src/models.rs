//! BoringSSL TLS state transition model

use std::marker::PhantomData;

/// So far we will only consider TLS streaming sockets.
pub enum TlsMode {}

/// Initial state of a client.
pub struct ClientInit<App>(PhantomData<fn() -> App>);

/// Earliest point of time in a server to do last minute configurations.
pub struct ClientHello<App: ?Sized>(PhantomData<fn() -> App>);

impl<App> Configurable<ClientInit<App>> for TlsMode {
    type Config = ConfigForClientInit;
}
impl<App> Configurable<ClientHello<App>> for TlsMode {
    type Config = ConfigForClientHello<App>;
}
