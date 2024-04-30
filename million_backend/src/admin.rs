use entity::crawler_queue;
use proto::admin::{AddUrlToQueueRequest, AddUrlToQueueResponse};
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseConnection};
use tonic::{Response, Status};

pub struct AdminServise {
    pub db: DatabaseConnection,
}

#[tonic::async_trait]
impl proto::admin::admin_server::Admin for AdminServise {
    async fn add_url_to_queue(
        &self,
        request: tonic::Request<AddUrlToQueueRequest>,
    ) -> std::result::Result<tonic::Response<AddUrlToQueueResponse>, tonic::Status> {
        let request = request.into_inner();

        let add_to_queue = crawler_queue::ActiveModel {
            url: ActiveValue::Set(request.url),
            statis: ActiveValue::Set(String::from("queued")),

            ..Default::default()
        };

        let _in_queue = add_to_queue
            .insert(&self.db)
            .await
            .map_err(|err| Status::from_error(err.into()))?;

        Ok(Response::new(AddUrlToQueueResponse{}))
    }
}
