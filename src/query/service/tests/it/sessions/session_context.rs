//  Copyright 2021 Datafuse Labs.
//
//  Licensed under the Apache License, Version 2.0 (the "License");
//  you may not use this file except in compliance with the License.
//  You may obtain a copy of the License at
//
//      http://www.apache.org/licenses/LICENSE-2.0
//
//  Unless required by applicable law or agreed to in writing, software
//  distributed under the License is distributed on an "AS IS" BASIS,
//  WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//  See the License for the specific language governing permissions and
//  limitations under the License.

use std::net::SocketAddr;

use common_base::base::tokio;
use common_exception::Result;
use common_meta_types::UserInfo;
use common_settings::Settings;
use databend_query::sessions::SessionContext;
use databend_query::Config;

#[tokio::test]
async fn test_session_context() -> Result<()> {
    let conf = Config::load()?;
    let tenant = &conf.query.tenant_id;
    let settings = Settings::default_settings(tenant);
    let session_ctx = SessionContext::try_create(conf, settings)?;

    // Abort status.
    {
        session_ctx.set_abort(true);
        let val = session_ctx.get_abort();
        assert!(val);
    }

    // Current database status.
    {
        session_ctx.set_current_database("bend".to_string());
        let val = session_ctx.get_current_database();
        assert_eq!("bend", val);
    }

    // Client host.
    {
        let demo = "127.0.0.1:80";
        let server: SocketAddr = demo.parse().unwrap();
        session_ctx.set_client_host(Some(server));

        let val = session_ctx.get_client_host();
        assert_eq!(Some(server), val);
    }

    // Current user.
    {
        let user_info = UserInfo::new_no_auth("user1", "");
        session_ctx.set_current_user(user_info);

        let val = session_ctx.get_current_user().unwrap();
        assert_eq!("user1".to_string(), val.name);
    }

    // io shutdown tx.
    {
        let (tx, _) = futures::channel::oneshot::channel();
        session_ctx.set_io_shutdown_tx(Some(tx));

        let val = session_ctx.take_io_shutdown_tx();
        assert!(val.is_some());

        let val = session_ctx.take_io_shutdown_tx();
        assert!(val.is_none());
    }

    Ok(())
}
