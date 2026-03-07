use async_trait::async_trait;
use barqflow_core::errors::BarqError;
use barqflow_core::schema::INodeExecutionData;
use barqflow_core::traits::{IExecuteFunctions, INodeType};
use barqflow_core::types::IDataObject;

pub struct SalesforceNode;

impl SalesforceNode {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl INodeType for SalesforceNode {
    fn get_description(&self) -> IDataObject {
        IDataObject::new()
    }

    async fn execute(
        &self,
        _context: &dyn IExecuteFunctions,
    ) -> Result<Vec<Vec<INodeExecutionData>>, BarqError> {
        Ok(vec![vec![INodeExecutionData::new(IDataObject::new())]])
    }
}
