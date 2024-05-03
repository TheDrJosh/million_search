use std::{io::Cursor, str::FromStr};

use chrono::{Duration, NaiveDateTime, Utc};
use entity::{crawler_queue, websites};
use proto::{
    crawler::{
        return_job_request::{
            self,
            return_job_request_ok::{image::Data, Body},
        },
        GetJobRequest, GetJobResponse, KeepAliveJobRequest, KeepAliveJobResponse, ReturnJobRequest,
        ReturnJobResponse,
    },
    tonic::{self, Response, Status},
};
use s3::Bucket;
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, Condition, DatabaseConnection, EntityTrait,
    QueryFilter,
};
use url::Url;

#[derive(Debug)]
pub struct CrawlerServise {
    pub db: DatabaseConnection,
    pub bucket: Bucket,
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

        let result = if let Some(result) = request.result {
            if let return_job_request::Result::Ok(result) = result {
                result
            } else {
                //TODO: increment attemps
                return Ok(Response::new(ReturnJobResponse {}));
            }
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
            Some(Body::Html(html_body)) => {
                let website = websites::ActiveModel {
                    url: ActiveValue::Set(request.url),
                    mime_type: ActiveValue::Set(result.mime_type),
                    icon_url: ActiveValue::Set(html_body.icon_url),
                    ..Default::default()
                };
                website
                    .insert(&self.db)
                    .await
                    .map_err(|err| Status::from_error(err.into()))?;
            }
            Some(Body::Image(image_body)) => match image_body.data {
                Some(Data::Bitmap(bitmap)) => {
                    let img = image::io::Reader::new(Cursor::new(bitmap.data))
                        .with_guessed_format()
                        .map_err(|err| Status::from_error(err.into()))?
                        .decode()
                        .map_err(|err| Status::from_error(err.into()))?;

                    let img = if img.width() > 256 || img.height() > 256 {
                        img.resize(256, 256, image::imageops::FilterType::Gaussian)
                    } else {
                        img
                    };

                    let mut img_av1 = Vec::new();

                    img.write_to(&mut Cursor::new(&mut img_av1), image::ImageFormat::Avif)
                        .map_err(|err| Status::from_error(err.into()))?;

                    self.bucket
                        .put_object(request.url, &img_av1)
                        .await
                        .map_err(|err| Status::from_error(err.into()))?;
                }

                Some(Data::Vector(vector)) => {
                    todo!()
                }
                None => {}
            },
            Some(Body::Video(_video_body)) => {
                todo!()
            }
            Some(Body::Audio(_audio_body)) => {
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
