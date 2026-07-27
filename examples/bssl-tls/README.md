# State-First Design for BoringSSL Rust Bindings

This example models the protocol as a typed transition system in Rust, where each connection phase is represented by a distinct type. A connection is carried as `TlsConnection<Mode, State>`, and each state implements `TransitState` so the next phase is encoded in the type system rather than hidden in runtime flags.

## TL;DR

This is a demonstration of the following points.

- Field projection provides zero-cost safe-guard in the configuration space, without introducing runtime data moves.
- Aspect-oriented programming can be enabled by a more flexible `Receiver::Target` design, where method dispatches to new code without further changes to the upstream TLS library.

## Why this design is useful

Transport Layer Security, or TLS for short, is never a simple free-standing cryptographic-enabled communication protocol. As the protocol evolves, it is endowed with a lot of configuration options that are unsafe or meaningless to change when the handshake protocol progresses past certain states. The configuration space is also so vast that it will be costly to move the data between `enum` variants.

A recurring anti-pattern in TLS library design is the inverted control of the protocol. Libraries with today's design tend to focus on API that is protocol-centric: the configuration is set ahead of the handshake and the connection is established in one-shot. This is a convenient model for clients but it is hostile to server applications even when moderate level of control needs to be imposed from the application side. Frequently are the cases where a server application needs to respond or behave differently in response to the client configuration, availability of server resources and permissions, decision made by the TLS stack, etc. The state transition is mostly opaque to server application with exception of callbacks from the TLS stack back to the application.

In this design, we will return the control of the state transition back to application clients and servers, with a safe API to expose the part of configuration that is most sensible to change at which state. The API design will also be application-centric, resulting in an API that is easier to be customised by application, with minimal changes to the TLS library itself.

### Compile-time safety for protocol transitions

The implementation distinguishes states such as `ClientInit<App>`, `ClientHello<App>`, `ServerHello<App>`, and `ClientOfferCredential<App>`. Because each transition is implemented only for the appropriate state, invalid transitions are rejected at compile time instead of being discovered later at runtime.

### Explicit access to state-specific configuration

The connection's configuration is not treated as one bag of buttons and sliders. Instead, `Configurable<State>` selects a suitable configuration type for a given state, and `ProjectPlace` lets the code borrow a specific subfield such as `tls_config`, `client_hello`, or `credentials` through typed handles. This keeps state-specific access explicit and avoids ad-hoc mutation. For instance, once the connection has transitioned into the credential-selection `ClientOfferCredential` state, it is too late to change the cipher suite because that decision is already locked in by the earlier handshake state.

Most importantly, this is **zero-cost**. The configuration space can be pacted into one object and the access control is selected by the state. There is no more need to juggle the configuration state between `enum` variants and everything can be pinned in the memory.

### Clear separation of immutable and mutable state

The example models different applicable configuration space for different phases.

- In the `ServerInit` phase, the config is still being prepared, so it is reasonable for the code to mutate `tls_config` before the connection advances to the next state.
- In the `ClientHello` phase, the handshake data and TLS settings are now fixed as inputs to the protocol flow, so `tls_config` and `client_hello` are projected through immutable `RefHandle`s. The `credentials` field is different because the signing logic must update its internal state as it progresses, so it is projected through a `MutHandle`.

This design makes the borrow path explicit and reduces the chance of accidental aliasing or invalid mutation on a protocol's perspective.

### State transition in a readable language

As a small demonstration, this is how a state transition would look like. This is a clean design language, the TLS library only needs to focus on the state management according to a smaller part of [RFC 9846 Appendix A](https://www.rfc-editor.org/info/rfc9846/#appendix-A).

```rust
// TLS library

// Allowed transition: ClientInit<App> ===> ServerHello<App>
impl<App: InitClient> TransitState for ClientInit<App> {
    type Mode = TlsMode;
    type Next = ServerHello<App>;

    async fn poll_until_next(
        mut self: TlsConnection<Self::Mode, Self>,
    ) -> Result<TlsConnection<Self::Mode, Self::Next>, ()> {
        App::prepare_client_hello(@self.tls_config).await?;

        TlsConnection::transit::<Self::Next>(self)
    }
}
```

The design allows us to write TLS connection state transition in the application code in a readable prose, thanks to the `Receiver` trait implementation.

```rust
// From server application's point of view:
let conn: TlsConnection<TlsMode, ServerInit> = ..;
// == Server is ready to accept a handshake ==
let conn: TlsConnection<TlsMode, ClientHello> = conn.poll_until_next().await?;
// == Server gets ClientHello ==
let conn: TlsConnection<TlsMode, ServerSelectCertificate> = conn.poll_until_next().await?;
// == .. ==
```

`conn.poll_until_next()` naturally follows a subject-verb order and it is unambiguous as to which `poll_until_next` function is to be called on.

## Future work

In order for a more intuitive "TLS configuration language" to work, we would need to invest more into a more free `Receiver` trait definition, so that the restriction on `Receiver::Target` is lifted. When that happens, this trait definition works.

```rust
trait InitClient {
    async fn prepare_client_hello(self: &mut TlsConnection<TlsMode, Self>) -> Result<(), ()>;
}
```

The consequence would be Rust would allow dot-method calls can be resolved per typestate. From the downstream users' point of view, the application is at the central of the API design and the customisation of the TLS protocol should be placed under the application type context.

```rust
// So in a downtream crate one could configure this trait...
impl InitClient for MyAppClient {
    async fn prepare_client_hello(self: &mut TlsConnection<TlsMode, Self>) -> Result<(), ()>
    {
        // Options
        // - Negotiate ALPN HTTP2 per application specification.
        // - Offer credentials depending on availability of keys in the hardware key-store.
        // - ...
    }
}

// This reads naturally when `App` in question is clear from the context.
conn.prepare_client_hello()?;
```
