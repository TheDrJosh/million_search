use std::str::FromStr;

use chrono::{Duration, NaiveDateTime, Utc};
use entity::{crawler_queue, websites};
use meilisearch_sdk::Client;
use proto::{
    crawler::{
        return_job_request::{self},
        GetJobRequest, GetJobResponse, KeepAliveJobRequest, KeepAliveJobResponse, ReturnJobRequest,
        ReturnJobResponse,
    },
    tonic::{self, Response, Status},
}; 
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, Condition, DatabaseConnection, EntityTrait,
    QueryFilter,
};
use url::Url;

use crate::search::WebsiteSearch;

#[derive(Debug)]
pub struct CrawlerServise {
    pub db: DatabaseConnection,
    pub search_client: Client,
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

        crawler_queue::Entity::update_many()
            .col_expr(crawler_queue::Column::Status, "executing".into())
            .col_expr(
                crawler_queue::Column::LastUpdated,
                chrono::Utc::now().naive_utc().into(),
            )
            .col_expr(
                crawler_queue::Column::Expiry,
                Some((chrono::Utc::now() + Duration::minutes(5)).naive_utc()).into(),
            )
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

        let result = if let Some(return_job_request::Result::Ok(result)) = request.result {
            result
        } else {
            //TODO: increment attemps
            return Ok(Response::new(ReturnJobResponse {}));
        };

        let _url = Url::from_str(&request.url).map_err(|err| Status::from_error(err.into()))?;

        let task = crawler_queue::Entity::find_by_id(request.id)
            .filter(crawler_queue::Column::Url.eq(request.url.clone()))
            .one(&self.db)
            .await
            .map_err(|err| Status::from_error(err.into()))?
            .ok_or(Status::invalid_argument("task not found"))?;

        if task.status != "executing" {
            return Err(Status::invalid_argument("not an active task"));
        }

        if task.expiry.unwrap() < Utc::now().naive_utc() {
            return Err(Status::invalid_argument("task expired"));
        }

        crawler_queue::Entity::update_many()
            .col_expr(crawler_queue::Column::Status, "complete".into())
            .col_expr(
                crawler_queue::Column::Expiry,
                Option::<NaiveDateTime>::None.into(),
            )
            .col_expr(
                crawler_queue::Column::LastUpdated,
                chrono::Utc::now().naive_utc().into(),
            )
            .filter(crawler_queue::Column::Id.eq(request.id))
            .filter(crawler_queue::Column::Url.eq(request.url.clone()))
            .exec(&self.db)
            .await
            .map_err(|err| Status::from_error(err.into()))?;

        for url in result.linked_urls {
            let mut url = url
                .parse::<Url>()
                .map_err(|err| Status::from_error(err.into()))?;

            url.set_fragment(None);

            if crawler_queue::Entity::find()
                .filter(crawler_queue::Column::Url.eq(url.to_string()))
                .all(&self.db)
                .await
                .map_err(|err| Status::from_error(err.into()))?
                .len()
                != 0
            {
                continue;
            }

            let website = crawler_queue::ActiveModel {
                url: ActiveValue::Set(url.to_string()),
                status: ActiveValue::Set(String::from("queued")),
                ..Default::default()
            };
            website
                .insert(&self.db)
                .await
                .map_err(|err| Status::from_error(err.into()))?;
        }

        match result.body {
            Some(return_job_request::ok::Body::Html(html_body)) => {
                let website = websites::ActiveModel {
                    url: ActiveValue::Set(request.url),
                    title: ActiveValue::Set(html_body.title),
                    description: ActiveValue::Set(html_body.description),
                    icon_url: ActiveValue::Set(html_body.icon_url),

                    text_fields: ActiveValue::Set(html_body.text_fields),
                    sections: ActiveValue::Set(html_body.sections),

                    ..Default::default()
                };
                let website = website
                    .insert(&self.db)
                    .await
                    .map_err(|err| Status::from_error(err.into()))?;

                self.search_client
                    .index("websites")
                    .add_documents(
                        &[WebsiteSearch {
                            id: website.id as i64,
                            url: website.url,
                            title: website.title,
                            description: website.description,
                            text_fields: website.text_fields,
                            sections: website.sections,
                        }],
                        None,
                    )
                    .await
                    .map_err(|err| Status::from_error(err.into()))?;
            }
            Some(return_job_request::ok::Body::Image(image_body)) => {
                // let image = image::ActiveModel {
                //     url: ActiveValue::Set(request.url),
                //     title: ActiveValue::Set(html_body.title),
                //     description: ActiveValue::Set(html_body.description),
                //     icon_url: ActiveValue::Set(html_body.icon_url),

                //     text_fields: ActiveValue::Set(html_body.text_fields),
                //     sections: ActiveValue::Set(html_body.sections),

                //     ..Default::default()
                // };
                todo!()
            }
            Some(return_job_request::ok::Body::Video(video_body)) => {
                todo!()
            }
            Some(return_job_request::ok::Body::Audio(audio_body)) => {
                todo!()
            }
            None => {}
        }

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
