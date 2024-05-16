use entity::crawler_queue;
use proto::{
    admin::{
        AddUrlToQueueRequest, AddUrlToQueueResponse, GetAllUrlsInQueueRequest,
        GetAllUrlsInQueueResponse,
    },
    tonic::{self, Response, Status},
};
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
};
use entity::sea_orm_active_enums::Status as JobStatus;

#[derive(Debug)]
pub struct AdminServise {
    pub db: DatabaseConnection,
}

#[tonic::async_trait]
impl proto::admin::admin_server::Admin for AdminServise {
    async fn add_url_to_queue(
        &self,
        request: tonic::Request<AddUrlToQueueRequest>,
    ) -> Result<tonic::Response<AddUrlToQueueResponse>, tonic::Status> {
        let request = request.into_inner();

        let add_to_queue = crawler_queue::ActiveModel {
            url: ActiveValue::Set(request.url),
            status: ActiveValue::Set(JobStatus::Queued),

            ..Default::default()
        };

        let _in_queue = add_to_queue
            .insert(&self.db)
            .await
            .map_err(|err| Status::from_error(err.into()))?;

        Ok(Response::new(AddUrlToQueueResponse {}))
    }

    async fn get_all_urls_in_queue(
        &self,
        _request: tonic::Request<GetAllUrlsInQueueRequest>,
    ) -> Result<tonic::Response<GetAllUrlsInQueueResponse>, tonic::Status> {
        let urls = crawler_queue::Entity::find()
            .filter(crawler_queue::Column::Status.ne(JobStatus::Complete))
            .all(&self.db)
            .await
            .map_err(|err| Status::from_error(err.into()))?
            .into_iter()
            .map(|queue_item| queue_item.url)
            .collect();

        Ok(Response::new(GetAllUrlsInQueueResponse { urls }))
    }
}
