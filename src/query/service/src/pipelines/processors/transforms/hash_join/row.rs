// Copyright 2022 Datafuse Labs.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::hash::Hash;
use std::hash::Hasher;
use std::sync::RwLock;

use common_datablocks::DataBlock;
use common_datablocks::KeysState;
use common_datavalues::ColumnRef;
use common_datavalues::DataSchemaRef;
use common_exception::Result;

use crate::pipelines::processors::transforms::hash_join::join_hash_table::MarkerKind;

pub type ColumnVector = Vec<ColumnRef>;

pub struct Chunk {
    pub data_block: DataBlock,
    pub cols: ColumnVector,
    pub keys_state: Option<KeysState>,
}

impl Chunk {
    pub fn num_rows(&self) -> usize {
        self.data_block.num_rows()
    }
}

#[derive(Clone, Copy, Debug)]
pub struct RowPtr {
    pub chunk_index: u32,
    pub row_index: u32,
    pub marker: Option<MarkerKind>,
}

pub struct RowSpace {
    pub data_schema: DataSchemaRef,
    pub chunks: RwLock<Vec<Chunk>>,
}

impl RowSpace {
    pub fn new(data_schema: DataSchemaRef) -> Self {
        Self {
            data_schema,
            chunks: RwLock::new(vec![]),
        }
    }

    pub fn schema(&self) -> DataSchemaRef {
        self.data_schema.clone()
    }

    pub fn push_cols(&self, data_block: DataBlock, cols: ColumnVector) -> Result<()> {
        let chunk = Chunk {
            data_block,
            cols,
            keys_state: None,
        };

        {
            // Acquire write lock in current scope
            let mut chunks = self.chunks.write().unwrap();
            chunks.push(chunk);
        }

        Ok(())
    }

    pub fn datablocks(&self) -> Vec<DataBlock> {
        let chunks = self.chunks.read().unwrap();
        chunks.iter().map(|c| c.data_block.clone()).collect()
    }

    pub fn rows_number(&self) -> usize {
        let chunks = self.chunks.read().unwrap();
        chunks.iter().map(|c| c.num_rows()).sum()
    }

    pub fn gather(&self, row_ptrs: &[RowPtr]) -> Result<DataBlock> {
        let data_blocks = self.datablocks();
        let num_rows = data_blocks
            .iter()
            .fold(0, |acc, block| acc + block.num_rows());
        let mut indices = Vec::with_capacity(row_ptrs.len());

        for row_ptr in row_ptrs.iter() {
            indices.push((
                row_ptr.chunk_index as usize,
                row_ptr.row_index as usize,
                1usize,
            ));
        }

        if !data_blocks.is_empty() && num_rows != 0 {
            let data_block =
                DataBlock::block_take_by_chunk_indices(&data_blocks, indices.as_slice())?;
            Ok(data_block)
        } else {
            Ok(DataBlock::empty_with_schema(self.data_schema.clone()))
        }
    }
}

impl PartialEq for RowPtr {
    fn eq(&self, other: &Self) -> bool {
        self.chunk_index == other.chunk_index && self.row_index == other.row_index
    }
}

impl Eq for RowPtr {}

impl Hash for RowPtr {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.chunk_index.hash(state);
        self.row_index.hash(state);
    }
}
