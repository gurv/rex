use std::{
//     collections::{HashMap, HashSet},
    // path::{Path, PathBuf},
    path::{PathBuf},
//     sync::{
//         Arc,
//     },
};

// use anyhow::{Context as _, ensure};
use anyhow::{Context as _};
use clap::Args;
// use hyper::server::conn::http1;
// use rustls::{ServerConfig, pki_types::CertificateDer};
// use rustls_pemfile::{certs, private_key};
// use tokio::{
//     net::TcpListener,
//     select,
//     sync::{RwLock},
// };
// use tokio_rustls::TlsAcceptor;
// use tracing::{debug, error};
use tracing::{error};
// use wasmcloud_runtime::component::CustomCtxComponent;
// use wasmtime::{AsContextMut, StoreContextMut, component::InstancePre};
use wasmtime::{StoreContextMut, component::InstancePre};
// use wasmtime_wasi::{DirPerms, FilePerms, WasiCtxBuilder};
use wasmtime_wasi_http::{
    WasiHttpView as _,
    bindings::{ProxyPre, http::types::Scheme},
    body::HyperOutgoingBody,
    // io::TokioIo,
};

use crate::{
    cli::{
        CliCommand, CliContext, CommandOutput,
    },
    runtime::{
        Ctx,
    },
};

/// Helper function to check if a path should be ignored during file watching
/// to prevent artifact directories from triggering rebuilds
///
/// # Arguments
/// * `path` - The file path to check
/// * `canonical_project_root` - The canonicalized project root directory
/// * `ignore_paths` - Set of canonicalized paths that should be ignored
// fn is_ignored(
//     path: &Path,
//     _canonical_project_root: &Path,
//     ignore_paths: &HashSet<PathBuf>,
// ) -> bool {
//     let canonical_path = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
//     ignore_paths.iter().any(|p| canonical_path.starts_with(p))
// }

#[derive(Debug, Clone, Args)]
pub struct DevCommand {
    /// The path to the project directory
    #[clap(name = "project-dir", default_value = ".")]
    pub project_dir: PathBuf,

    /// The path to the built Wasm file to be used in development
    #[clap(long = "artifact-path")]
    pub artifact_path: Option<PathBuf>,

    /// The address on which the HTTP server will listen
    #[clap(long = "address", default_value = "0.0.0.0:8000")]
    pub address: String,

    /// Configuration values to use for `wasi:config/runtime` in the form of `key=value` pairs.
    #[clap(long = "runtime-config", value_delimiter = ',')]
    pub runtime_config: Vec<String>,

    /// The root directory for the blobstore to use for `wasi:blobstore/blobstore`. Defaults to a subfolder in the rex data directory.
    #[clap(long = "blobstore-root")]
    pub blobstore_root: Option<PathBuf>,

    /// Path to TLS certificate file (PEM format) for HTTPS support
    #[clap(long = "tls-cert", requires = "tls_key")]
    pub tls_cert: Option<PathBuf>,

    /// Path to TLS private key file (PEM format) for HTTPS support
    #[clap(long = "tls-key", requires = "tls_cert")]
    pub tls_key: Option<PathBuf>,

    /// Path to CA certificate bundle (PEM format) for client certificate verification (optional)
    #[clap(long = "tls-ca")]
    pub tls_ca: Option<PathBuf>,
}

impl CliCommand for DevCommand {
    async fn handle(&self, _ctx: &CliContext) -> anyhow::Result<CommandOutput> {
        Ok(CommandOutput::ok(
            "Development command executed successfully".to_string(),
            None,
        ))
    }
}

/// Load TLS configuration from certificate and key files
// async fn load_tls_config(
//     cert_path: &Path,
//     key_path: &Path,
//     ca_path: Option<&Path>,
// ) -> anyhow::Result<ServerConfig> {
//     // Load certificate chain
//     let cert_data = tokio::fs::read(cert_path)
//         .await
//         .with_context(|| format!("Failed to read certificate file: {}", cert_path.display()))?;
//     let mut cert_reader = std::io::Cursor::new(cert_data);
//     let cert_chain: Vec<CertificateDer<'static>> = certs(&mut cert_reader)
//         .collect::<Result<Vec<_>, _>>()
//         .with_context(|| format!("Failed to parse certificate file: {}", cert_path.display()))?;

//     ensure!(
//         !cert_chain.is_empty(),
//         "No certificates found in file: {}",
//         cert_path.display()
//     );

//     // Load private key
//     let key_data = tokio::fs::read(key_path)
//         .await
//         .with_context(|| format!("Failed to read private key file: {}", key_path.display()))?;
//     let mut key_reader = std::io::Cursor::new(key_data);
//     let key = private_key(&mut key_reader)
//         .with_context(|| format!("Failed to parse private key file: {}", key_path.display()))?
//         .ok_or_else(|| anyhow::anyhow!("No private key found in file: {}", key_path.display()))?;

//     // Create rustls server config
//     let config = ServerConfig::builder()
//         .with_no_client_auth()
//         .with_single_cert(cert_chain, key)
//         .with_context(|| "Failed to create TLS configuration")?;

//     // If CA is provided, configure client certificate verification
//     if let Some(ca_path) = ca_path {
//         let ca_data = tokio::fs::read(ca_path)
//             .await
//             .with_context(|| format!("Failed to read CA file: {}", ca_path.display()))?;
//         let mut ca_reader = std::io::Cursor::new(ca_data);
//         let ca_certs: Vec<CertificateDer<'static>> = certs(&mut ca_reader)
//             .collect::<Result<Vec<_>, _>>()
//             .with_context(|| format!("Failed to parse CA file: {}", ca_path.display()))?;

//         ensure!(
//             !ca_certs.is_empty(),
//             "No CA certificates found in file: {}",
//             ca_path.display()
//         );

//         // Note: Client certificate verification configuration would go here
//         // For now, we'll keep it simple without client cert verification
//         debug!("CA certificate loaded, but client certificate verification not yet implemented");
//     }

//     Ok(config)
// }

/// Starts the development HTTP server, listening for incoming requests
/// and serving them using the provided [`CustomCtxComponent`]. The component
/// is provided via a `tokio::sync::watch::Receiver`, allowing it to be
/// updated dynamically (e.g. on a rebuild).
// async fn server(
//     rx: &mut tokio::sync::watch::Receiver<Arc<CustomCtxComponent<Ctx>>>,
//     address: String,
//     runtime_config: HashMap<String, String>,
//     blobstore_root: PathBuf,
//     background_processes: Arc<RwLock<Vec<tokio::process::Child>>>,
//     tls_acceptor: Option<TlsAcceptor>,
// ) -> anyhow::Result<()> {
//     // Prepare our server state and start listening for connections.
//     let mut component = rx.borrow_and_update().to_owned();
//     let listener = TcpListener::bind(&address).await?;
//     loop {
//         let blobstore_root = blobstore_root.clone();
//         let runtime_config = runtime_config.clone();
//         let background_processes = background_processes.clone();
//         select! {
//             // If the component changed, replace the current one
//             _ = rx.changed() => {
//                 // If the channel has changed, we need to update the component
//                 component = rx.borrow_and_update().to_owned();
//                 debug!("Component updated in main loop");
//             }
//             // Accept a TCP connection and serve all of its requests in a separate
//             // tokio task. Note that for now this only works with HTTP/1.1.
//             Ok((client, addr)) = listener.accept() => {
//                 let component = component.clone();
//                 let background_processes = background_processes.clone();
//                 let tls_acceptor = tls_acceptor.clone();
//                 debug!(addr = ?addr, "serving new client");

//                 tokio::spawn(async move {
//                     let component = component.clone();

//                     // Determine the scheme based on whether TLS is configured
//                     let scheme = if tls_acceptor.is_some() {
//                         Scheme::Https
//                     } else {
//                         Scheme::Http
//                     };

//                     // Handle TLS if configured
//                     let service = hyper::service::service_fn(move |req| {
//                         let component = component.clone();
//                         let background_processes = background_processes.clone();
//                         let scheme = scheme.clone();
//                         let wasi_ctx = match WasiCtxBuilder::new()
//                             .preopened_dir(&blobstore_root, "/dev", DirPerms::all(), FilePerms::all())
//                         {
//                             Ok(ctx) => ctx.build(),
//                             Err(e) => {
//                                 error!(err = ?e, "failed to create WASI context with preopened dir");
//                                 WasiCtxBuilder::new().build()
//                             }
//                         };
//                         let ctx = Ctx::builder()
//                             .with_wasi_ctx(wasi_ctx)
//                             .with_runtime_config(runtime_config.clone())
//                             .with_background_processes(background_processes)
//                             .build();
//                         async move { component.handle_request(Some(ctx), req, scheme).await }
//                     });

//                     let result = if let Some(acceptor) = tls_acceptor {
//                         // Handle HTTPS connection
//                         match acceptor.accept(client).await {
//                             Ok(tls_stream) => {
//                                 http1::Builder::new()
//                                     .keep_alive(true)
//                                     .serve_connection(TokioIo::new(tls_stream), service)
//                                     .await
//                             }
//                             Err(e) => {
//                                 error!(addr = ?addr, err = ?e, "TLS handshake failed");
//                                 return;
//                             }
//                         }
//                     } else {
//                         // Handle HTTP connection
//                         http1::Builder::new()
//                             .keep_alive(true)
//                             .serve_connection(TokioIo::new(client), service)
//                             .await
//                     };

//                     if let Err(e) = result {
//                         error!(addr = ?addr, err = ?e, "error serving client");
//                     }
//                 });
//             }
//         }
//     }
// }

/// Simple trait for handling an HTTP request, used primarily to extend the
/// `CustomCtxComponent` with a method that can handle HTTP requests
// trait HandleRequest {
//     async fn handle_request(
//         &self,
//         ctx: Option<Ctx>,
//         req: hyper::Request<hyper::body::Incoming>,
//         scheme: Scheme,
//     ) -> anyhow::Result<hyper::Response<HyperOutgoingBody>>;
// }

// impl HandleRequest for CustomCtxComponent<Ctx> {
//     async fn handle_request(
//         &self,
//         ctx: Option<Ctx>,
//         req: hyper::Request<hyper::body::Incoming>,
//         scheme: Scheme,
//     ) -> anyhow::Result<hyper::Response<HyperOutgoingBody>> {
//         // Create per-http-request state within a `Store` and prepare the
//         // initial resources passed to the `handle` function.
//         let ctx = ctx.unwrap_or_default();
//         let mut store = self.new_store(ctx);
//         let pre = self.instance_pre().clone();
//         handle_request(store.as_context_mut(), pre, req, scheme).await
//     }
// }

pub async fn handle_request<'a>(
    mut store: StoreContextMut<'a, Ctx>,
    pre: InstancePre<Ctx>,
    req: hyper::Request<hyper::body::Incoming>,
    scheme: Scheme,
) -> anyhow::Result<hyper::Response<HyperOutgoingBody>> {
    let (sender, receiver) = tokio::sync::oneshot::channel();
    let req = store.data_mut().new_incoming_request(scheme, req)?;
    let out = store.data_mut().new_response_outparam(sender)?;
    let pre = ProxyPre::new(pre).context("failed to instantiate proxy pre")?;

    // Run the http request itself in a separate task so the task can
    // optionally continue to execute beyond after the initial
    // headers/response code are sent.
    // let task: JoinHandle<anyhow::Result<()>> = tokio::spawn(async move {
    let proxy = pre.instantiate_async(&mut store).await?;

    proxy
        .wasi_http_incoming_handler()
        .call_handle(&mut store, req, out)
        .await?;

    // Ok(())
    // });

    match receiver.await {
        // If the client calls `response-outparam::set` then one of these
        // methods will be called.
        Ok(Ok(resp)) => Ok(resp),
        Ok(Err(e)) => Err(e.into()),

        // Otherwise the `sender` will get dropped along with the `Store`
        // meaning that the oneshot will get disconnected and here we can
        // inspect the `task` result to see what happened
        Err(e) => {
            error!(err = ?e, "error receiving http response");
            Err(anyhow::anyhow!(
                "oneshot channel closed but no response was sent"
            ))
            // Err(match task.await {
            //     Ok(Ok(())) => {
            //         anyhow::anyhow!("oneshot channel closed but no response was sent")
            //     }
            //     Ok(Err(e)) => e,
            //     Err(e) => {
            //         anyhow::anyhow!("failed to await task for handling HTTP request: {e}")
            //     }
            // })
        }
    }
}
