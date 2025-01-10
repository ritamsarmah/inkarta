create table if not exists images (
    id integer primary key,
    title text not null,
    artist text,
    background integer not null,
    data blob not null,
    thumbnail blob not null
);
