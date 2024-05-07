FROM postgres:16.2

RUN apt-get update \
    && apt-get install -y postgresql-16-wal2json

RUN echo "wal_level = logical" >> /usr/share/postgresql/postgresql.conf.sample