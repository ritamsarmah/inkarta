create table if not exists images (
    id integer primary key,
    title text not null,
    artist text not null,
    dark boolean not null,
    data blob not null
);
