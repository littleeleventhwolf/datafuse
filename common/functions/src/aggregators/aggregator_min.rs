// Copyright 2020-2021 The Datafuse Authors.
//
// SPDX-License-Identifier: Apache-2.0.

use std::fmt;

use common_exception::{Result, ErrorCodes};
use common_datablocks::DataBlock;
use common_datavalues::DataArrayAggregate;
use common_datavalues::DataColumnarValue;
use common_datavalues::DataSchema;
use common_datavalues::DataType;
use common_datavalues::DataValue;
use common_datavalues::DataValueAggregate;
use common_datavalues::DataValueAggregateOperator;

use crate::IFunction;

#[derive(Clone)]
pub struct AggregatorMinFunction {
    depth: usize,
    arg: Box<dyn IFunction>,
    state: DataValue
}

impl AggregatorMinFunction {
    pub fn try_create(args: &[Box<dyn IFunction>]) -> Result<Box<dyn IFunction>> {
        match args.len() {
            1 => {
                Ok(Box::new(AggregatorMinFunction {
                    depth: 0,
                    arg: args[0].clone(),
                    state: DataValue::Null,
                }))
            }
            _ => Result::Err(ErrorCodes::BadArguments("Function Error: Aggregator function Min args require single argument".to_string()))
        }
    }
}

impl IFunction for AggregatorMinFunction {
    fn name(&self) -> &str {
        "AggregatorMinFunction"
    }

    fn return_type(&self, input_schema: &DataSchema) -> Result<DataType> {
        self.arg.return_type(input_schema)
    }

    fn nullable(&self, _input_schema: &DataSchema) -> Result<bool> {
        Ok(false)
    }

    fn eval(&self, block: &DataBlock) -> Result<DataColumnarValue> {
        self.arg.eval(block)
    }

    fn set_depth(&mut self, depth: usize) {
        self.depth = depth;
    }

    fn accumulate(&mut self, block: &DataBlock) -> Result<()> {
        let rows = block.num_rows();
        let val = self.arg.eval(&block)?;
        self.state = DataValueAggregate::data_value_aggregate_op(
            DataValueAggregateOperator::Min,
            self.state.clone(),
            DataArrayAggregate::data_array_aggregate_op(
                DataValueAggregateOperator::Min,
                val.to_array(rows)?,
            )?,
        )?;
        Ok(())
    }

    fn accumulate_result(&self) -> Result<Vec<DataValue>> {
        Ok(vec![self.state.clone()])
    }

    fn merge(&mut self, states: &[DataValue]) -> Result<()> {
        let val = states[self.depth].clone();
        self.state = DataValueAggregate::data_value_aggregate_op(
            DataValueAggregateOperator::Min,
            self.state.clone(),
            val,
        )?;
        Ok(())
    }

    fn merge_result(&self) -> Result<DataValue> {
        Ok(self.state.clone())
    }

    fn is_aggregator(&self) -> bool {
        true
    }
}

impl fmt::Display for AggregatorMinFunction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "min({})", self.arg)
    }
}
