use entity::{search_history, websites};
use futures::future::join_all;
use meilisearch_sdk::{Client, SearchResults};
use proto::{
    search::{
        CompleteSearchRequest, CompleteSearchResponse, SearchAudioRequest, SearchAudioResponse,
        SearchImageRequest, SearchImageResponse, SearchVideoRequest, SearchVideoResponse,
        SearchWebRequest, SearchWebResponse, SearchWebResult,
    },
    tonic::{self, Response, Status},
};
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct WebsiteSearch {
    pub id: i64,
    pub url: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub text_fields: Vec<String>,
    pub sections: Vec<String>,
}

#[derive(Debug)]
pub struct SearchServise {
    pub db: DatabaseConnection,
    pub search_client: Client,
}

#[tonic::async_trait]
impl proto::search::search_server::Search for SearchServise {
    async fn complete_search(
        &self,
        request: tonic::Request<CompleteSearchRequest>,
    ) -> std::result::Result<tonic::Response<CompleteSearchResponse>, tonic::Status> {
        let request = request.into_inner();

        let partial_query = request.current;

        let results: SearchResults<SearchHistory> = self
            .search_client
            .index("search_history")
            .search()
            .with_query(&partial_query)
            .execute()
            .await
            .map_err(|err| Status::from_error(err.into()))?;

        let res: Vec<String> = results
            .hits
            .into_iter()
            .map(|search| search.result.text)
            .collect();

        return Ok(Response::new(CompleteSearchResponse { possibilities: res }));
    }

    async fn search_web(
        &self,
        request: tonic::Request<SearchWebRequest>,
    ) -> std::result::Result<tonic::Response<SearchWebResponse>, tonic::Status> {
        let request = request.into_inner();

        let query = request
            .query
            .ok_or(Status::invalid_argument("must have query"))?;

        save_search_to_history(&self.db, query.query.clone())
            .await
            .map_err(|err| Status::from_error(err.into()))?;

        let result: SearchResults<Websites> = self
            .search_client
            .index("websites")
            .search()
            .with_query(&query.query)
            .execute()
            .await
            .map_err(|err| Status::from_error(err.into()))?;

        let list = join_all(
            result
                .hits
                .iter()
                .skip(query.start as usize)
                .take(query.length as usize)
                .map(|web| websites::Entity::find_by_id(web.result.id as i32).one(&self.db)),
        )
        .await
        .into_iter()
        .collect::<Result<Option<Vec<_>>, _>>()
        .map_err(|err| Status::from_error(err.into()))?
        .ok_or(Status::internal("desync between postgres and meiliseach"))?;

        let results = list
            .into_iter()
            .map(|model| SearchWebResult {
                url: model.url,
                title: model.title,
                description: model.description,
                icon_url: model.icon_url,
                inner_text_match: None,
            })
            .collect::<Vec<_>>();

        Ok(Response::new(SearchWebResponse { results }))
    }

    async fn search_image(
        &self,
        request: tonic::Request<SearchImageRequest>,
    ) -> std::result::Result<tonic::Response<SearchImageResponse>, tonic::Status> {
        let request = request.into_inner();
        let query = request
            .query
            .ok_or(Status::invalid_argument("must have query"))?;

        save_search_to_history(&self.db, query.query)
            .await
            .map_err(|err| Status::from_error(err.into()))?;
        todo!()
    }

    async fn search_video(
        &self,
        request: tonic::Request<SearchVideoRequest>,
    ) -> std::result::Result<tonic::Response<SearchVideoResponse>, tonic::Status> {
        let request = request.into_inner();
        let query = request
            .query
            .ok_or(Status::invalid_argument("must have query"))?;

        save_search_to_history(&self.db, query.query)
            .await
            .map_err(|err| Status::from_error(err.into()))?;
        todo!()
    }
    async fn search_audio(
        &self,
        request: tonic::Request<SearchAudioRequest>,
    ) -> std::result::Result<tonic::Response<SearchAudioResponse>, tonic::Status> {
        let request = request.into_inner();
        let query = request
            .query
            .ok_or(Status::invalid_argument("must have query"))?;

        save_search_to_history(&self.db, query.query)
            .await
            .map_err(|err| Status::from_error(err.into()))?;
        todo!()
    }
}

async fn save_search_to_history(db: &DatabaseConnection, search: String) -> anyhow::Result<()> {
    let search = search_history::ActiveModel {
        text: sea_orm::ActiveValue::Set(search),
        ..Default::default()
    };
    search.insert(db).await?;
    Ok(())
}

#[derive(Serialize, Deserialize)]
struct Websites {
    id: i64,
    url: String,
    // title: Option<String>,
    // description: Option<String>,
    // text_fields: Vec<String>,
    // sections: Vec<String>,
}

#[derive(Serialize, Deserialize)]
struct Image {
    id: i64,
    url: String,
}

#[derive(Serialize, Deserialize)]
struct Video {
    id: i64,
    url: String,
}

#[derive(Serialize, Deserialize)]
struct Audio {
    id: i64,
    url: String,
}

#[derive(Serialize, Deserialize)]
struct SearchHistory {
    id: i64,
    text: String,
}
