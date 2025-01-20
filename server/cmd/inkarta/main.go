package main

import (
	"fmt"
	"inkarta/internal/db"
	"inkarta/internal/routes"
	"log/slog"
	"net/http"
)

const port = 5000

func main() {
	queries, err := db.Connect()
	if err != nil {
		panic(err)
	}
	defer db.Close()

	slog.Info("Starting server", "port", port)

	addr := fmt.Sprintf(":%v", port)
	router := routes.NewRouter(queries)

	if err := http.ListenAndServe(addr, router); err != nil {
		panic(err)
	}
}
