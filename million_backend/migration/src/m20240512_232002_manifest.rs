use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Manifest::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Manifest::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Manifest::UrlDomain).string().not_null())
                    .col(ColumnDef::new(Manifest::Name).string())
                    .col(ColumnDef::new(Manifest::ShortName).string())
                    .col(ColumnDef::new(Manifest::Description).string())
                    .col(
                        ColumnDef::new(Manifest::Categories)
                            .array(ColumnType::String(None))
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Manifest::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Manifest {
    Table,
    Id,
    UrlDomain,
    Name,
    ShortName,
    Description,
    Categories,
}
