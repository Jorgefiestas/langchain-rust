use std::collections::VecDeque;
use std::fs::File;

use std::io::{BufRead, BufReader, Result};
use std::path::Path;
use std::pin::Pin;
use std::task::{Context, Poll};

use futures::stream::{Stream, StreamExt};

use crate::document_loaders::Loader;
use crate::schemas::Document;
use crate::text_splitter::TextSplitter;

pub struct TxtLoader {
    reader: BufReader<File>,
    memory_queue: VecDeque<Document>,
}

impl TxtLoader {
    pub fn new(filepath: &str) -> Result<Self> {
        let path = Path::new(filepath);
        let file = File::open(&path)?;

        Ok(Self {
            reader: BufReader::new(file),
            memory_queue: VecDeque::new(),
        })
    }
}

impl Stream for TxtLoader {
    type Item = Result<Document>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if let Some(doc_ref) = self.memory_queue.front() {
            let doc = doc_ref.clone();
            self.memory_queue.pop_front();
            return Poll::Ready(Some(Ok(doc)));
        }

        let mut line = String::new();
        if self.reader.read_line(&mut line)? == 0 {
            return Poll::Ready(None);
        }

        let mut line = String::new();
        match self.reader.read_line(&mut line) {
            Err(e) => Poll::Ready(Some(Err(e.into()))),
            Ok(0) => Poll::Ready(None),
            Ok(_) => {
                self.memory_queue = self.split_text(line);
                self.poll_next(cx)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_txt_loader() {
        let path = "./src/document_loaders/test_data/test.csv";
        let mut txt_loader = TxtLoader::new(path).unwrap();

        while let Some(line) = txt_loader.next().await {
            println!("{:?}", line)
        }
    }
}
