use amqprs::{
    channel::{BasicPublishArguments, Channel},
    BasicProperties,
};
use proto::admin::{AddUrlToQueueRequest, AddUrlToQueueResponse};
use tonic::Status;

use crate::{EXCHANGE_NAME, ROUTING_KEY};

pub struct AdminServise {
    pub channel: Channel,
}

#[tonic::async_trait]
impl proto::admin::admin_server::Admin for AdminServise {
    async fn add_url_to_queue(
        &self,
        request: tonic::Request<AddUrlToQueueRequest>,
    ) -> std::result::Result<tonic::Response<AddUrlToQueueResponse>, tonic::Status> {
        let inner = request.into_inner();
        let url = inner.url;

        let args = BasicPublishArguments::new(EXCHANGE_NAME, ROUTING_KEY);

        self.channel
            .basic_publish(BasicProperties::default(), url.into_bytes(), args)
            .await
            .map_err(|err| Status::from_error(err.into()))?;

        todo!()
    }
}
