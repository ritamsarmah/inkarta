package main

import (
	"context"
	"database/sql"
	_ "embed"
	"fmt"
	"html/template"
	"inkarta/internal/database"
	"log"
	"log/slog"
	"net/http"
	"strconv"
	"time"

	_ "modernc.org/sqlite"
)

const dsnURI = "file:inkarta.db?cache=shared&mode=rwc&journal_mode=WAL"

//go:embed schema.sql
var ddl string
var db *sql.DB
var queries *database.Queries

func main() {
	if err := initDatabase(); err != nil {
		log.Fatal("Failed to initialize database with error:", err)
	}
	defer closeDatabase()

	http.HandleFunc("GET /", homePage)
	http.HandleFunc("GET /upload", uploadPage)
	http.HandleFunc("GET /view/{id}", viewPage)

	http.HandleFunc("GET /device/rtc", rtc)
	http.HandleFunc("GET /device/alarm", alarm)

	// http.HandleFunc("POST /image", createImage)
	// http.HandleFunc("GET /image/{id}", getImage)
	// http.HandleFunc("DELETE /image/{id}", deleteImage)
	// http.HandleFunc("GET /image/next", getNextImage)
	// http.HandleFunc("PUT /image/next/{id}", putNextImage)

	slog.Info("Starting Inkarta server...")
	log.Fatal(http.ListenAndServe(":5000", nil))
}

/* Database */

func initDatabase() error {
	ctx := context.Background()
	var err error

	slog.Info("Initializing database...")

	// Open database
	db, err = sql.Open("sqlite", dsnURI)
	if err != nil {
		return err
	}

	// Create tables
	if _, err = db.ExecContext(ctx, ddl); err != nil {
		return err
	}

	// Initialize database queries
	queries = database.New(db)

	return err
}

func closeDatabase() {
	if err := db.Close(); err != nil {
		slog.Error("Failed to close database", "error", err)
	}
}

/* Pages */

func homePage(w http.ResponseWriter, _ *http.Request) {
	ctx := context.Background()

	images, err := queries.ListImages(ctx)
	if err != nil {
		slog.Error("Failed to fetch list of images", "error", err)
	}

	tmpl := template.Must(template.ParseFiles("templates/base.html", "templates/index.html"))
	tmpl.Execute(w, images)
}

func uploadPage(w http.ResponseWriter, _ *http.Request) {
	tmpl := template.Must(template.ParseFiles("templates/base.html", "templates/upload.html"))
	tmpl.Execute(w, nil)
}

func viewPage(w http.ResponseWriter, r *http.Request) {
	ctx := context.Background()

	id, err := strconv.ParseInt(r.PathValue("id"), 10, 64)
	if err != nil {
		// TODO: Handle by routing to not found page
	}

	image, err := queries.GetImage(ctx, id)
	if err != nil {
		// TODO: Handle by routing to not found page
	}

	tmpl := template.Must(template.ParseFiles("templates/base.html", "templates/upload.html"))
	tmpl.Execute(w, image)
}

/* Device */

// Returns Unix epoch timestamp in server's timezone for device RTC.
func rtc(w http.ResponseWriter, _ *http.Request) {
	timestamp := time.Now().Unix()
	slog.Debug("Returning timestamp for real-time clock", "timestamp", timestamp)

	fmt.Fprint(w, timestamp)
}

// Returns Unix epoch timestamp for next display refresh (i.e., at midnight in server's timezone)
func alarm(w http.ResponseWriter, _ *http.Request) {
	tomorrow := time.Now().AddDate(0, 0, 1)
	midnight := time.Date(tomorrow.Year(), tomorrow.Month(), tomorrow.Day(), 0, 0, 0, 0, tomorrow.Location())
	timestamp := midnight.Unix()

	fmt.Fprint(w, timestamp)
}

/* Image */

// func getImage(w http.ResponseWriter, _ *http.Request) {
// 	db.
// }
