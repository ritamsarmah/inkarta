package db

import (
	"context"
	"database/sql"
	_ "embed"
	"log/slog"
	_ "modernc.org/sqlite"
)

const dsnURI = "file:inkarta.db"

//go:embed schema.sql
var ddl string
var db *sql.DB

func Connect() (*Queries, error) {
	var err error

	db, err = sql.Open("sqlite", dsnURI)
	if err != nil {
		return nil, err
	}

	ctx := context.Background()
	if _, err := db.ExecContext(ctx, ddl); err != nil {
		return nil, err
	}

	return New(db), err
}

func Close() {
	if err := db.Close(); err != nil {
		slog.Error("Failed to close database", "error", err)
	}
}
