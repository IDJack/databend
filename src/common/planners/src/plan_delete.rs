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

use common_datavalues::DataSchema;
use common_datavalues::DataSchemaRef;
use common_meta_app::schema::TableIdent;

use crate::Expression;
use crate::Projection;

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
pub struct DeletePlan {
    pub catalog_name: String,
    pub database_name: String,
    pub table_name: String,
    pub table_id: TableIdent,
    pub selection: Option<Expression>,
    pub projection: Projection,
}

impl DeletePlan {
    pub fn schema(&self) -> DataSchemaRef {
        Arc::new(DataSchema::empty())
    }
}
