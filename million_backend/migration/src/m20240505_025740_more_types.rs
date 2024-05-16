use sea_orm_migration::prelude::*;

use crate::m20220101_000001_create_table::Websites;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Image::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Image::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Image::Url).string().not_null())
                    .col(ColumnDef::new(Image::Source).integer().not_null())
                    .col(ColumnDef::new(Image::Width).integer())
                    .col(ColumnDef::new(Image::Height).integer())
                    .col(ColumnDef::new(Image::AltText).string())
                    .col(
                        ColumnDef::new(Image::CreatedAt)
                            .timestamp()
                            .default(Expr::current_timestamp())
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                ForeignKey::create()
                    .from(Image::Table, Image::Source)
                    .to(Websites::Table, Websites::Id)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Image::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum Image {
    Table,
    Id,
    Url,
    Width,
    Height,
    AltText,
    Source,
    CreatedAt,
}
