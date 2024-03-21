use std::collections::VecDeque;
use std::fs::File;

use std::io::{BufRead, BufReader};
use std::path::Path;
use std::pin::Pin;
use std::task::{Context, Poll};

use async_trait::async_trait;
use futures::stream::{Stream, StreamExt};

use crate::document_loaders::{Loader, LoaderError};
use crate::schemas::Document;
use crate::text_splitter::{ChunkSplitter, TextSplitter, TokenSplitter};

pub struct TxtLoader {
    filepath: String,
}

impl TxtLoader {
    pub fn new() -> Self {
        TxtLoader {
            filepath: String::new(),
        }
    }

    pub fn with_filepath(mut self, filepath: &str) -> Self {
        self.filepath = filepath.to_string();
        self
    }
}

#[async_trait]
impl Loader for TxtLoader {
    async fn load(self) -> Result<TxtLoaderStream, LoaderError> {
        TxtLoaderStream::new(&self.filepath)
    }

    async fn load_and_split<TS: TextSplitter>(
        self,
        splitter: TS,
    ) -> Result<TxtLoaderStream<TS>, LoaderError> {
        Ok(TxtLoaderStream::new(&self.filepath)?.with_splitter(Box::new(splitter)))
    }
}

pub struct TxtLoaderStream<TS: TextSplitter = TokenSplitter> {
    reader: BufReader<File>,
    splitter: Option<Box<TS>>,
    memory_queue: VecDeque<Document>,
}

impl<TS: TextSplitter> TxtLoaderStream<TS> {
    pub fn new(filepath: &str) -> Result<Self, LoaderError> {
        let path = Path::new(filepath);
        let file = File::open(&path)?;

        Ok(Self {
            reader: BufReader::new(file),
            splitter: None,
            memory_queue: VecDeque::new(),
        })
    }

    pub fn with_splitter(mut self, splitter_ptr: Box<TS>) -> Self {
        self.splitter = Some(splitter_ptr);
        self
    }
}

impl<TS: TextSplitter> Stream for TxtLoaderStream<TS> {
    type Item = Result<Document, LoaderError>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if let Some(doc_ref) = self.memory_queue.front() {
            let doc = doc_ref.clone();
            self.memory_queue.pop_front();
            return Poll::Ready(Some(Ok(doc)));
        }

        let mut line = String::new();
        match self.reader.read_line(&mut line) {
            Ok(0) => Poll::Ready(None),
            Ok(_) => {
                if self.splitter.is_none() {
                    return Poll::Ready(Some(Ok(Document::new(line))));
                }

                let documents: Vec<Document> = self
                    .splitter
                    .as_ref()
                    .unwrap()
                    .split_text(&line)?
                    .into_iter()
                    .map(Document::new)
                    .collect();

                self.memory_queue = VecDeque::from(documents);
                self.poll_next(cx)
            }
            Err(e) => Poll::Ready(Some(Err(e.into()))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_txt_loader() {
        let path = "./src/document_loaders/test_data/test.csv";
        let txt_loader = TxtLoader::new().with_filepath(path);

        let splitter = ChunkSplitter::new(2, false);

        let mut documents = txt_loader.load_and_split(splitter).await.unwrap();

        while let Some(document) = documents.next().await {
            println!("{:?}", document)
        }
    }
}
