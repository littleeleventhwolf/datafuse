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
use common_meta_types::TenantQuota;
use common_users::UserApiProvider;
use databend_query::interpreters::InterpreterFactory;
use databend_query::sessions::TableContext;
use databend_query::sql::*;
use futures::StreamExt;
use pretty_assertions::assert_eq;

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_user_stage_interpreter() -> Result<()> {
    let (_guard, ctx) = crate::tests::create_query_context().await?;
    let mut planner = Planner::new(ctx.clone());

    // add
    {
        let query = "CREATE STAGE test_stage url='s3://load/files/' credentials=(aws_key_id='1a2b3c' aws_secret_key='4x5y6z')";
        let (plan, _, _) = planner.plan_sql(query).await?;
        let executor = InterpreterFactory::get(ctx.clone(), &plan).await?;
        assert_eq!(executor.name(), "CreateUserStageInterpreter");
        let mut stream = executor.execute(ctx.clone()).await?;
        while let Some(_block) = stream.next().await {}
    }

    // desc
    {
        let query = "DESC STAGE test_stage";
        let (plan, _, _) = planner.plan_sql(query).await?;
        let executor = InterpreterFactory::get(ctx.clone(), &plan).await?;
        let mut stream = executor.execute(ctx.clone()).await?;
        let mut blocks = vec![];

        while let Some(block) = stream.next().await {
            blocks.push(block?);
        }

        common_datablocks::assert_blocks_eq(
            vec![
                "+------------+------------+-----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------+-------------------------------------------------------------+--------------------------------------------------------------------------------------------------------------------+-----------------+--------------------+---------+",
                "| name       | stage_type | stage_params                                                                                                                                                                                                                                                                                              | copy_options                                                | file_format_options                                                                                                | number_of_files | creator            | comment |",
                "+------------+------------+-----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------+-------------------------------------------------------------+--------------------------------------------------------------------------------------------------------------------+-----------------+--------------------+---------+",
                "| test_stage | External   | StageParams { storage: S3(StorageS3Config { endpoint_url: \"https://s3.amazonaws.com\", region: \"\", bucket: \"load\", root: \"/files/\", disable_credential_loader: true, enable_virtual_host_style: false, access_key_id: \"******b3c\", secret_access_key: \"******y6z\", security_token: \"\", master_key: \"\" }) } | CopyOptions { on_error: None, size_limit: 0, purge: false } | FileFormatOptions { format: Csv, skip_header: 0, field_delimiter: \",\", record_delimiter: \"\\n\", compression: None } | NULL            | 'root'@'127.0.0.1' |         |",
                "+------------+------------+-----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------+-------------------------------------------------------------+--------------------------------------------------------------------------------------------------------------------+-----------------+--------------------+---------+",
            ],
            &blocks,
        );
    }

    let tenant = ctx.get_tenant();
    let user_mgr = UserApiProvider::instance();
    let stage = user_mgr.get_stage(&tenant, "test_stage").await;
    assert!(stage.is_ok());

    // quota limit
    {
        let quota_api = user_mgr.get_tenant_quota_api_client(&tenant)?;
        let quota = TenantQuota {
            max_stages: 1,
            ..Default::default()
        };
        quota_api.set_quota(&quota, None).await?;
        let query = "CREATE STAGE test_stage url='s3://load/files/' credentials=(aws_key_id='1a2b3c' aws_secret_key='4x5y6z')";
        let (plan, _, _) = planner.plan_sql(query).await?;
        let executor = InterpreterFactory::get(ctx.clone(), &plan).await?;
        assert_eq!(executor.name(), "CreateUserStageInterpreter");
        let res = executor.execute(ctx.clone()).await;
        assert!(res.is_err());
        assert_eq!(
            res.err().unwrap().to_string(),
            "Code: 2903, displayText = Max stages quota exceeded 1."
        )
    }

    // drop
    {
        let query = "DROP STAGE if exists test_stage";
        let (plan, _, _) = planner.plan_sql(query).await?;
        let executor = InterpreterFactory::get(ctx.clone(), &plan).await?;
        assert_eq!(executor.name(), "DropUserStageInterpreter");

        let mut stream = executor.execute(ctx.clone()).await?;
        while let Some(_block) = stream.next().await {}
    }

    let stage = user_mgr.get_stage(&tenant, "test_stage").await;
    assert!(stage.is_err());
    Ok(())
}
