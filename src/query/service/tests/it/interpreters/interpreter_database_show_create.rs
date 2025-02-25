// Copyright 2021 Datafuse Labs.
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

use common_base::base::tokio;
use common_exception::Result;
use databend_query::interpreters::*;
use databend_query::sql::Planner;
use futures::TryStreamExt;
use pretty_assertions::assert_eq;

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_show_create_database_interpreter() -> Result<()> {
    let (_guard, ctx) = crate::tests::create_query_context().await?;
    let mut planner = Planner::new(ctx.clone());

    // show create database default
    {
        let query = "show create database default";
        let (plan, _, _) = planner.plan_sql(query).await?;
        let executor = InterpreterFactory::get(ctx.clone(), &plan).await?;
        assert_eq!(executor.name(), "ShowCreateDatabaseInterpreter");
        let stream = executor.execute(ctx.clone()).await?;
        let result = stream.try_collect::<Vec<_>>().await?;
        let expected = vec![
            "+----------+---------------------------+",
            "| Database | Create Database           |",
            "+----------+---------------------------+",
            "| default  | CREATE DATABASE `default` |",
            "+----------+---------------------------+",
        ];
        common_datablocks::assert_blocks_sorted_eq(expected, result.as_slice());
    }

    // show create database system
    {
        let query = "show create database system";
        let (plan, _, _) = planner.plan_sql(query).await?;
        let executor = InterpreterFactory::get(ctx.clone(), &plan).await?;
        assert_eq!(executor.name(), "ShowCreateDatabaseInterpreter");
        let stream = executor.execute(ctx.clone()).await?;
        let result = stream.try_collect::<Vec<_>>().await?;
        let expected = vec![
            "+----------+----------------------------------------+",
            "| Database | Create Database                        |",
            "+----------+----------------------------------------+",
            "| system   | CREATE DATABASE `system` ENGINE=SYSTEM |",
            "+----------+----------------------------------------+",
        ];
        common_datablocks::assert_blocks_sorted_eq(expected, result.as_slice());
    }

    Ok(())
}
