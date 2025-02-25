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

use std::sync::Arc;

use common_arrow::arrow::bitmap::MutableBitmap;
use common_exception::ErrorCode;
use common_exception::Result;

use crate::columns::mutable::MutableColumn;
use crate::prelude::*;

#[derive(Debug)]
pub struct MutableObjectColumn<T>
where T: ObjectType
{
    pub(crate) values: Vec<T>,
}

impl<T> MutableColumn for MutableObjectColumn<T>
where T: ObjectType
{
    fn data_type(&self) -> DataTypeImpl {
        T::data_type()
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_mut_any(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn append_default(&mut self) {
        self.append_value(T::default());
    }

    fn validity(&self) -> Option<&MutableBitmap> {
        None
    }

    fn shrink_to_fit(&mut self) {
        self.values.shrink_to_fit();
    }

    fn len(&self) -> usize {
        self.values.len()
    }

    fn to_column(&mut self) -> crate::ColumnRef {
        self.shrink_to_fit();
        Arc::new(ObjectColumn::<T> {
            values: std::mem::take(&mut self.values),
        })
    }

    fn append_data_value(&mut self, value: DataValue) -> Result<()> {
        let v: T = DFTryFrom::try_from(value)?;
        self.append_value(v);
        Ok(())
    }

    fn pop_data_value(&mut self) -> Result<DataValue> {
        let t = self.pop_value().ok_or_else(|| {
            ErrorCode::BadDataArrayLength("Object column array is empty when pop data value")
        })?;

        let data_value = DataValue::try_from(t)?;
        Ok(data_value)
    }
}

impl<T> Default for MutableObjectColumn<T>
where T: ObjectType
{
    fn default() -> Self {
        Self::with_capacity(0)
    }
}

impl<T> MutableObjectColumn<T>
where T: ObjectType
{
    pub fn from_data(values: Vec<T>) -> Self {
        Self { values }
    }

    pub fn append_value(&mut self, val: T) {
        self.values.push(val);
    }

    pub fn pop_value(&mut self) -> Option<T> {
        self.values.pop()
    }

    pub fn values(&self) -> &Vec<T> {
        &self.values
    }

    pub fn with_capacity(capacity: usize) -> Self {
        MutableObjectColumn {
            values: Vec::<T>::with_capacity(capacity),
        }
    }
}

impl<T> ScalarColumnBuilder for MutableObjectColumn<T>
where
    T: ObjectType,
    T: Scalar<ColumnType = ObjectColumn<T>>,
{
    type ColumnType = ObjectColumn<T>;

    fn with_capacity(capacity: usize) -> Self {
        MutableObjectColumn {
            values: Vec::<T>::with_capacity(capacity),
        }
    }

    fn with_capacity_meta(capacity: usize, _meta: ColumnMeta) -> Self {
        Self::with_capacity(capacity)
    }

    fn push(&mut self, value: <T as Scalar>::RefType<'_>) {
        self.values.push(value.to_owned_scalar());
    }

    fn finish(&mut self) -> Self::ColumnType {
        self.shrink_to_fit();
        ObjectColumn::<T> {
            values: std::mem::take(&mut self.values),
        }
    }
}
