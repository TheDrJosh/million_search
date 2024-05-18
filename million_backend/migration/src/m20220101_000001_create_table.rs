use sea_orm_migration::{
    prelude::*,
    sea_orm::{EnumIter, Iterable},
    sea_query::extension::postgres::{Type, TypeDropStatement},
};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Websites::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Websites::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Websites::Url).string().not_null())
                    .col(ColumnDef::new(Websites::Title).string())
                    .col(ColumnDef::new(Websites::Description).string())
                    .col(ColumnDef::new(Websites::IconUrl).string())
                    .col(
                        ColumnDef::new(Websites::TextFields)
                            .array(ColumnType::String(None))
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Websites::Sections)
                            .array(ColumnType::String(None))
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Websites::Keywords)
                            .array(ColumnType::String(None))
                            .not_null(),
                    )
                    .col(ColumnDef::new(Websites::SiteName).string())
                    .col(ColumnDef::new(Websites::SiteShortName).string())
                    .col(ColumnDef::new(Websites::SiteDescription).string())
                    .col(
                        ColumnDef::new(Websites::SiteCategories)
                            .array(ColumnType::String(None))
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Websites::CreatedAt)
                            .timestamp()
                            .default(Expr::current_timestamp())
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_type(
                Type::create()
                    .as_enum(Status)
                    .values(StatusVariants::iter())
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(CrawlerQueue::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(CrawlerQueue::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(CrawlerQueue::Url)
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .col(
                        ColumnDef::new(CrawlerQueue::Status)
                            .custom(Status)
                            .not_null(),
                    )
                    .col(ColumnDef::new(CrawlerQueue::Expiry).timestamp())
                    .col(
                        ColumnDef::new(CrawlerQueue::LastUpdated)
                            .timestamp()
                            .default(Expr::current_timestamp())
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(CrawlerQueue::CreatedAt)
                            .timestamp()
                            .default(Expr::current_timestamp())
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Websites::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(CrawlerQueue::Table).to_owned())
            .await?;
        manager
            .drop_type(TypeDropStatement::new().name(Status).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
pub enum Websites {
    Table,
    Id,
    Url,

    Title,
    Description,
    IconUrl,

    TextFields,
    Sections,
    Keywords,

    SiteName,
    SiteShortName,
    SiteDescription,
    SiteCategories,

    CreatedAt,
}

#[derive(DeriveIden)]
enum CrawlerQueue {
    Table,
    Id,
    Url,
    Status,
    Expiry,
    LastUpdated,
    CreatedAt,
}

#[derive(DeriveIden, EnumIter)]
enum StatusVariants {
    Queued,
    Complete,
    Executing,
}

#[derive(DeriveIden)]
struct Status;
