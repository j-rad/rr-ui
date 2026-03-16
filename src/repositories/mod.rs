#[cfg(feature = "server")]
pub mod balancer;
pub mod inbound;
#[cfg(feature = "server")]
pub mod routing;
pub mod setting;
pub mod user;

#[cfg(feature = "server")]
pub use balancer::{BalancerRepository, SurrealBalancerRepository};
pub use inbound::{InboundRepository, SurrealInboundRepository};
#[cfg(feature = "server")]
pub use routing::{RoutingRepository, SurrealRoutingRepository};
pub use setting::{SettingRepository, SurrealSettingRepository};
pub use user::{SurrealUserRepository, UserRepository};
