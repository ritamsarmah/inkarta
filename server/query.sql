-- name: GetImage :one
select * from images
where id = ? limit 1;

-- name: GetRandomImage :one
select * from images
order by random() limit 1;

-- name: ListImages :many
select id, title, artist from images
order by artist;

-- name: CreateImage :one
insert into images (
    title, artist, background, data, thumbnail
) values (
    ?, ?, ?, ?, ?
)
returning *;

-- name: DeleteImage :exec
delete from images
where id = ?;
