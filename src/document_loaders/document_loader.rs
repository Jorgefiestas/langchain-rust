use async_trait::async_trait;
use futures::Stream;

use crate::schemas::Document;

use super::LoaderError;

#[async_trait]
pub trait Loader: Stream + Send + Sync {}
