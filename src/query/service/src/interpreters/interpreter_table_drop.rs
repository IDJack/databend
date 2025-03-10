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

use std::sync::Arc;

use common_exception::ErrorCode;
use common_exception::Result;
use common_meta_types::GrantObject;
use common_meta_types::UserPrivilegeType;
use common_planners::DropTablePlan;
use common_planners::TruncateTablePlan;
use common_streams::DataBlockStream;
use common_streams::SendableDataBlockStream;

use crate::interpreters::Interpreter;
use crate::sessions::QueryContext;
use crate::sessions::TableContext;
use crate::storages::view::view_table::VIEW_ENGINE;

pub struct DropTableInterpreter {
    ctx: Arc<QueryContext>,
    plan: DropTablePlan,
}

impl DropTableInterpreter {
    pub fn try_create(ctx: Arc<QueryContext>, plan: DropTablePlan) -> Result<Self> {
        Ok(DropTableInterpreter { ctx, plan })
    }
}

#[async_trait::async_trait]
impl Interpreter for DropTableInterpreter {
    fn name(&self) -> &str {
        "DropTableInterpreter"
    }

    async fn execute(&self) -> Result<SendableDataBlockStream> {
        let catalog_name = self.plan.catalog.as_str();
        let db_name = self.plan.database.as_str();
        let tbl_name = self.plan.table.as_str();
        let tbl = self
            .ctx
            .get_table(catalog_name, db_name, tbl_name)
            .await
            .ok();

        self.ctx
            .get_current_session()
            .validate_privilege(
                &GrantObject::Database(catalog_name.into(), db_name.into()),
                UserPrivilegeType::Drop,
            )
            .await?;

        if let Some(table) = &tbl {
            if table.get_table_info().engine() == VIEW_ENGINE {
                return Err(ErrorCode::UnexpectedError(format!(
                    "{}.{} is VIEW, please use `DROP VIEW {}.{}`",
                    &self.plan.database, &self.plan.table, &self.plan.database, &self.plan.table
                )));
            }
        };

        let catalog = self.ctx.get_catalog(catalog_name)?;
        catalog.drop_table(self.plan.clone().into()).await?;

        if let Some(tbl) = tbl {
            // if `plan.all`, truncate, then purge the historical data
            if self.plan.all {
                // errors of truncation are ignored
                let _ = tbl
                    .truncate(self.ctx.clone(), TruncateTablePlan {
                        catalog: self.plan.catalog.clone(),
                        database: self.plan.database.clone(),
                        table: self.plan.table.clone(),
                        purge: true,
                    })
                    .await;
            }
        }

        Ok(Box::pin(DataBlockStream::create(
            self.plan.schema(),
            None,
            vec![],
        )))
    }
}
