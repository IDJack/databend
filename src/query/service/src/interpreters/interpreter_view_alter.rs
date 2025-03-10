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

use std::collections::BTreeMap;
use std::sync::Arc;

use common_exception::ErrorCode;
use common_exception::Result;
use common_meta_app::schema::CreateTableReq;
use common_meta_app::schema::DropTableReq;
use common_meta_app::schema::TableMeta;
use common_meta_app::schema::TableNameIdent;
use common_meta_types::GrantObject;
use common_meta_types::UserPrivilegeType;
use common_planners::AlterViewPlan;
use common_streams::DataBlockStream;
use common_streams::SendableDataBlockStream;

use crate::interpreters::Interpreter;
use crate::sessions::QueryContext;
use crate::sessions::TableContext;
use crate::storages::view::view_table::VIEW_ENGINE;

pub struct AlterViewInterpreter {
    ctx: Arc<QueryContext>,
    plan: AlterViewPlan,
}

impl AlterViewInterpreter {
    pub fn try_create(ctx: Arc<QueryContext>, plan: AlterViewPlan) -> Result<Self> {
        Ok(AlterViewInterpreter { ctx, plan })
    }
}

#[async_trait::async_trait]
impl Interpreter for AlterViewInterpreter {
    fn name(&self) -> &str {
        "AlterViewInterpreter"
    }

    async fn execute(&self) -> Result<SendableDataBlockStream> {
        // check privilige
        self.ctx
            .get_current_session()
            .validate_privilege(
                &GrantObject::Database(self.plan.catalog.clone(), self.plan.database.clone()),
                UserPrivilegeType::Create,
            )
            .await?;

        // check whether view has exists
        if !self
            .ctx
            .get_catalog(&self.plan.catalog)?
            .list_tables(&self.plan.tenant, &self.plan.database)
            .await?
            .iter()
            .any(|table| {
                table.name() == self.plan.viewname.as_str()
                    && table.get_table_info().engine() == VIEW_ENGINE
            })
        {
            return Err(ErrorCode::ViewAlreadyExists(format!(
                "{}.{} view is not existed",
                self.plan.database, self.plan.viewname
            )));
        }

        self.alter_view().await
    }
}

impl AlterViewInterpreter {
    async fn alter_view(&self) -> Result<SendableDataBlockStream> {
        // drop view
        let catalog = self.ctx.get_catalog(&self.plan.catalog)?;
        let plan = DropTableReq {
            if_exists: true,

            name_ident: TableNameIdent {
                tenant: self.plan.tenant.clone(),
                db_name: self.plan.database.clone(),
                table_name: self.plan.viewname.clone(),
            },
        };
        catalog.drop_table(plan).await?;

        // create new view
        let mut options = BTreeMap::new();
        options.insert("query".to_string(), self.plan.subquery.clone());
        let plan = CreateTableReq {
            if_not_exists: true,
            name_ident: TableNameIdent {
                tenant: self.plan.tenant.clone(),
                db_name: self.plan.database.clone(),
                table_name: self.plan.viewname.clone(),
            },
            table_meta: TableMeta {
                engine: VIEW_ENGINE.to_string(),
                options,
                ..Default::default()
            },
        };
        catalog.create_table(plan).await?;

        Ok(Box::pin(DataBlockStream::create(
            self.plan.schema(),
            None,
            vec![],
        )))
    }
}
