use std::{collections::Vector, error::Error};

use async_trait::async_trait;
use futures::Stream;

use crate::{schemas::Document, text_splitter::TextSplitter};

#[async_trait]
pub trait Loader: Stream + Send + Sync {
    fn split_text(self, doc: Document) -> Result<Vec<Document>, LoaderError>;
}

impl Loader {
    fn split_text(self, s: String) -> Result<Vec<Document>> {
        return self.splitter.split_text(s);
    }
}
