use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts

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
                    .col(ColumnDef::new(Websites::MimeType).string().not_null())
                    .col(ColumnDef::new(Websites::IconUrl).string())
                    .col(ColumnDef::new(Websites::CreatedAt).timestamp().default(Expr::current_timestamp()))
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
                    .col(ColumnDef::new(CrawlerQueue::Url).string().not_null())
                    .col(ColumnDef::new(CrawlerQueue::Statis).string().not_null())
                    .col(ColumnDef::new(CrawlerQueue::LastKeepAlive).timestamp().default(Expr::current_timestamp()))
                    .col(ColumnDef::new(CrawlerQueue::LastUpdated).timestamp().default(Expr::current_timestamp()))
                    .col(ColumnDef::new(CrawlerQueue::CreatedAt).timestamp().default(Expr::current_timestamp()))
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts

        manager
            .drop_table(Table::drop().table(Websites::Table).to_owned())
            .await?;
        manager
        .drop_table(Table::drop().table(CrawlerQueue::Table).to_owned())
        .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum Websites {
    Table,
    Id,
    MimeType,
    IconUrl,
    CreatedAt,
}

#[derive(DeriveIden)]
enum CrawlerQueue {
    Table,
    Id,
    Url,
    Statis,
    LastKeepAlive,
    LastUpdated,
    CreatedAt,
}
