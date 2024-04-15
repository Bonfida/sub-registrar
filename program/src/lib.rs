use bonfida_utils::declare_id_with_central_state;

#[doc(hidden)]
pub mod entrypoint;
#[doc(hidden)]
pub mod error;
/// Program instructions and their CPI-compatible bindings
pub mod instruction;
/// Describes the different data structres that the program uses to encode state
pub mod state;

#[doc(hidden)]
pub(crate) mod processor;

#[allow(missing_docs)]
pub mod cpi;

#[allow(missing_docs)]
pub mod utils;

#[allow(missing_docs)]
pub mod revoke_unchecked;

declare_id_with_central_state!("2KkyPzjaAYaz2ojQZ9P3xYakLd96B5UH6a2isLaZ4Cgs");

#[cfg(not(feature = "no-entrypoint"))]
solana_security_txt::security_txt! {
    name: env!("CARGO_PKG_NAME"),
    project_url: "http://bonfida.org",
    contacts: "email:security@bonfida.com,link:https://twitter.com/bonfida",
    policy: "https://immunefi.com/bounty/bonfida",
    preferred_languages: "en"
}
