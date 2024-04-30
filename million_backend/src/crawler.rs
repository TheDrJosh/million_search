use entity::crawler_queue;
use proto::crawler::{
    GetJobRequest, GetJobResponse, KeepAliveJobRequest, KeepAliveJobResponse, ReturnJobRequest,
    ReturnJobResponse,
};
use sea_orm::{ColumnTrait, Condition, DatabaseConnection, EntityTrait, QueryFilter};
use tonic::Status;

#[derive(Debug, Default)]
pub struct CrawlerServise {
    pub db: DatabaseConnection,
}

#[tonic::async_trait]
impl proto::crawler::crawler_server::Crawler for CrawlerServise {
    async fn get_job(
        &self,
        request: tonic::Request<GetJobRequest>,
    ) -> std::result::Result<tonic::Response<GetJobResponse>, tonic::Status> {
        let _request = request.into_inner();

        todo!()
    }

    async fn return_job(
        &self,
        request: tonic::Request<ReturnJobRequest>,
    ) -> std::result::Result<tonic::Response<ReturnJobResponse>, tonic::Status> {
        let request = request.into_inner();

        let task = crawler_queue::Entity::find()
            .filter(
                Condition::any()
                    .add(crawler_queue::Column::Status.eq("queued"))
                    .add(
                        Condition::all()
                            .add(crawler_queue::Column::Status.eq("executing"))
                            .add(crawler_queue::Column::Expiry.lte(chrono::Utc::now().naive_utc())),
                    ),
            )
            .one(&self.db)
            .await
            .map_err(|err| Status::from_error(err.into()))?
            .ok_or(anyhow::anyhow!("No Jobs in queue"))
            .map_err(|err| Status::from_error(err.into()))?;

        

        todo!()
    }

    async fn keep_alive_job(
        &self,
        request: tonic::Request<KeepAliveJobRequest>,
    ) -> std::result::Result<tonic::Response<KeepAliveJobResponse>, tonic::Status> {
        let request = request.into_inner();

        todo!()
    }
}
