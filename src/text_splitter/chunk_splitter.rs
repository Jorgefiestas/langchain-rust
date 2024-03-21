use super::{SplitterOptions, TextSplitter, TextSplitterError};

#[derive(Debug, Clone)]
pub struct ChunkSplitter {
    chunk_size: usize,
    trim_chunks: bool,
}

impl Default for ChunkSplitter {
    fn default() -> Self {
        ChunkSplitter {
            chunk_size: 10,
            trim_chunks: true,
        }
    }
}

impl ChunkSplitter {
    pub fn new(chunk_size: usize, trim_chunks: bool) -> Self {
        ChunkSplitter {
            chunk_size,
            trim_chunks,
        }
    }

    fn split(&self, text: &str) -> Vec<String> {
        text.chars()
            .collect::<Vec<_>>()
            .chunks(self.chunk_size)
            .map(|chunk| {
                let chunk_str: String = chunk.iter().collect();
                if self.trim_chunks {
                    chunk_str.trim().to_string()
                } else {
                    chunk_str
                }
            })
            .collect()
    }
}

impl TextSplitter for ChunkSplitter {
    fn split_text(&self, text: &str) -> Result<Vec<String>, TextSplitterError> {
        Ok(self.split(text))
    }
}
