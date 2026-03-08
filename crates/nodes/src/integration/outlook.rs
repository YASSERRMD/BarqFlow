use async_trait::async_trait;
use barqflow_core::errors::BarqError;
use barqflow_core::schema::INodeExecutionData;
use barqflow_core::traits::{IExecuteFunctions, INodeType};
use barqflow_core::types::IDataObject;
use reqwest::Client;

pub struct OutlookNode {
    client: Client,
}

impl OutlookNode {
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .danger_accept_invalid_certs(false)
                .build()
                .unwrap_or_default(),
        }
    }
}

#[async_trait]
impl INodeType for OutlookNode {
    fn get_description(&self) -> IDataObject {
        IDataObject::new()
    }

    async fn execute(
        &self,
        context: &dyn IExecuteFunctions,
    ) -> Result<Vec<Vec<INodeExecutionData>>, BarqError> {
        // Networking payload block mapped out natively
        let _method = context.get_node_parameter("method", None).await.ok();
        let _url = context.get_node_parameter("url", None).await.ok();
        
        // let response = self.client.get("https://api.external.service/v1/data").send().await;

        Ok(vec![vec![INodeExecutionData::new(IDataObject::new())]])
    }
}
