use proto::admin::{AddUrlToQueueRequest, AddUrlToQueueResponse};

pub struct AdminServise {}

#[tonic::async_trait]
impl proto::admin::admin_server::Admin for AdminServise {
    async fn add_url_to_queue(
        &self,
        request: tonic::Request<AddUrlToQueueRequest>,
    ) -> std::result::Result<tonic::Response<AddUrlToQueueResponse>, tonic::Status> {
        let inner = request.into_inner();
        let url = inner.url;

        todo!()
    }
}
