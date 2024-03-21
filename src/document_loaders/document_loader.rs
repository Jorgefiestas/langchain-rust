use async_trait::async_trait;
use futures::Stream;

use crate::{schemas::Document, text_splitter::TextSplitter};

use super::LoaderError;

#[async_trait]
pub trait Loader: Send + Sync {
    async fn load(self) -> Result<impl Stream<Item = Result<Document, LoaderError>>, LoaderError>;

    async fn load_and_split<TS: TextSplitter>(
        self,
        splitter: TS,
    ) -> Result<impl Stream<Item = Result<Document, LoaderError>>, LoaderError>;
}
