use std::collections::VecDeque;
use std::fs::File;

use std::io::{BufRead, BufReader};
use std::path::Path;
use std::pin::Pin;
use std::task::{Context, Poll};

use futures::stream::{Stream, StreamExt};

use crate::document_loaders::{Loader, LoaderError};
use crate::schemas::Document;

pub struct TxtLoader {
    reader: BufReader<File>,
    memory_queue: VecDeque<Document>,
    split_fn: Option<fn(String) -> Vec<String>>,
}

impl TxtLoader {
    pub fn new(filepath: &str) -> Result<Self, LoaderError> {
        let path = Path::new(filepath);
        let file = File::open(&path)?;

        Ok(Self {
            reader: BufReader::new(file),
            memory_queue: VecDeque::new(),
            split_fn: None,
        })
    }

    pub fn with_split_fn(mut self, split_fn: fn(String) -> Vec<String>) -> Self {
        self.split_fn = Some(split_fn);
        self
    }
}

impl Loader for TxtLoader {}

impl Stream for TxtLoader {
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
                let documents = match self.split_fn {
                    Some(split_fn) => split_fn(line).into_iter().map(Document::new).collect(),
                    None => vec![Document::new(line)],
                };
                self.memory_queue.extend(documents);
                self.poll_next(cx)
            }
            Err(e) => Poll::Ready(Some(Err(e.into()))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn split_into_pairs(input: String) -> Vec<String> {
        input
            .chars() // Iterate over characters
            .collect::<Vec<char>>() // Collect characters into a Vec<char> to enable chunking
            .chunks(2) // Create an iterator over each chunk of 2 characters
            .map(|chunk| chunk.iter().collect()) // Collect each chunk back into a String
            .collect() // Collect all the Strings into a Vec<String>
    }

    #[tokio::test]
    async fn test_txt_loader() {
        let path = "./src/document_loaders/test_data/test.csv";
        let mut txt_loader = TxtLoader::new(path)
            .unwrap()
            .with_split_fn(split_into_pairs);

        while let Some(line) = txt_loader.next().await {
            println!("{:?}", line)
        }
    }
}
