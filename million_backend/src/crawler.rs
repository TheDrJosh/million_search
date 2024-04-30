use chrono::Duration;
use entity::{crawler_queue, websites};
use proto::{
    crawler::{
        GetJobRequest, GetJobResponse, KeepAliveJobRequest, KeepAliveJobResponse, ReturnJobRequest,
        ReturnJobResponse,
    },
    tonic::{self, Response, Status},
};
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, Condition, DatabaseConnection, EntityTrait,
    IntoActiveModel, QueryFilter,
};

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
            .ok_or(Status::resource_exhausted("No more Jobs in queue"))?;

        let mut active_task = task.clone().into_active_model();

        active_task.status = ActiveValue::Set(String::from("executing"));
        active_task.last_updated = ActiveValue::Set(chrono::Utc::now().naive_utc());
        active_task.expiry = ActiveValue::Set(Some(
            (chrono::Utc::now() + Duration::minutes(5)).naive_utc(),
        ));

        let task = crawler_queue::Entity::update(active_task)
            .filter(crawler_queue::Column::Id.eq(task.id))
            .filter(crawler_queue::Column::LastUpdated.eq(task.last_updated))
            .exec(&self.db)
            .await
            .map_err(|err| Status::from_error(err.into()))?;

        Ok(Response::new(GetJobResponse {
            id: task.id,
            url: task.url,
        }))
    }

    async fn return_job(
        &self,
        request: tonic::Request<ReturnJobRequest>,
    ) -> std::result::Result<tonic::Response<ReturnJobResponse>, tonic::Status> {
        let request = request.into_inner();

        // TODO: Check expire to accept input

        let task = crawler_queue::ActiveModel {
            status: ActiveValue::Set(String::from("complete")),
            expiry: ActiveValue::Set(None),
            last_updated: ActiveValue::Set(chrono::Utc::now().naive_utc()),
            ..Default::default()
        };

        crawler_queue::Entity::update(task)
            .filter(crawler_queue::Column::Id.eq(request.id))
            .filter(crawler_queue::Column::Url.eq(request.url.clone()))
            .exec(&self.db)
            .await
            // .unwrap();
            .map_err(|err| Status::from_error(err.into()))?;

        let website = websites::ActiveModel {
            url: ActiveValue::Set(request.url),
            mime_type: ActiveValue::Set(request.mime_type),
            icon_url: ActiveValue::Set(request.icon_url),
            ..Default::default()
        };
        website
            .insert(&self.db)
            .await
            // .unwrap();
            .map_err(|err| Status::from_error(err.into()))?;

        Ok(Response::new(ReturnJobResponse {}))
    }

    async fn keep_alive_job(
        &self,
        request: tonic::Request<KeepAliveJobRequest>,
    ) -> std::result::Result<tonic::Response<KeepAliveJobResponse>, tonic::Status> {
        let request = request.into_inner();

        let task = crawler_queue::ActiveModel {
            expiry: ActiveValue::Set(Some(
                (chrono::Utc::now() + Duration::minutes(5)).naive_utc(),
            )),
            last_updated: ActiveValue::Set(chrono::Utc::now().naive_utc()),
            ..Default::default()
        };

        crawler_queue::Entity::update(task)
            .filter(crawler_queue::Column::Id.eq(request.id))
            .filter(crawler_queue::Column::Url.eq(request.url.clone()))
            .exec(&self.db)
            .await
            .map_err(|err| Status::from_error(err.into()))?;

        Ok(Response::new(KeepAliveJobResponse {}))
    }
}
