use proto::admin::{AddUrlToQueueRequest, AddUrlToQueueResponse};

#[derive(Debug, Default)]
pub struct AdminServise {}

#[tonic::async_trait]
impl proto::admin::admin_server::Admin for AdminServise {
    async fn add_url_to_queue(
        &self,
        request: tonic::Request<AddUrlToQueueRequest>,
    ) -> std::result::Result<tonic::Response<AddUrlToQueueResponse>, tonic::Status> {
        todo!()
    }
}
