use amiquip::{Channel, Connection, Exchange, Publish};
use proto::admin::{AddUrlToQueueRequest, AddUrlToQueueResponse};
use tokio::sync::Mutex;
use tonic::transport::channel;

#[derive(Debug)]
pub struct AdminServise  {
    pub connection: Mutex<Connection>,
}

#[tonic::async_trait]
impl proto::admin::admin_server::Admin for AdminServise {
    async fn add_url_to_queue(
        &self,
        request: tonic::Request<AddUrlToQueueRequest>,
    ) -> std::result::Result<tonic::Response<AddUrlToQueueResponse>, tonic::Status> {

        let channel = self.connection.lock().await.open_channel(None).map_err(|err| tonic::Status::from_error(err.into()))?;

        let exchange = Exchange::direct(&channel);

        exchange.publish(Publish::new("hello there".as_bytes(), "Crawler Queue")).map_err(|err| tonic::Status::from_error(err.into()))?;

        todo!()
    }
}
