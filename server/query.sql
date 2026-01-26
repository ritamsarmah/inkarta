-- name: GetImage :one
select * from images
where id = ? limit 1;

-- name: GetImageInfo :one
select id, title, artist from images
where id = ? limit 1;

-- name: GetRandomImage :one
select * from images
order by random() limit 1;

-- name: ListImages :many
select * from images
order by artist;

-- name: ListImageInfo :many
select id, title, artist from images
order by artist;

-- name: CreateImage :exec
insert into images (
    title, artist, dark, data
) values (
    ?, ?, ?, ?
) returning id;

-- name: DeleteImage :exec
delete from images
where id = ?;
