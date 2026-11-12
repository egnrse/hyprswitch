//! sort clients; update clients with monitor/workspace data

pub use sort::sort_clients;
pub use update::update_clients;

#[allow(clippy::module_inception)]
mod sort;
mod update;

#[cfg(test)]
pub(super) mod tests;
