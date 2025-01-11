package main

import (
	"bytes"
	"context"
	"database/sql"
	"fmt"
	"html/template"
	"inkarta/internal/database"
	"log"
	"log/slog"
	"mime/multipart"
	"net/http"
	"strconv"
	"time"

	"golang.org/x/image/bmp"
	"golang.org/x/image/draw"
	"image"
	_ "image/gif"
	_ "image/jpeg"
	_ "image/png"

	_ "embed"
	_ "modernc.org/sqlite"
)

const dsnURI = "file:inkarta.db?cache=shared&mode=rwc&journal_mode=WAL"

//go:embed schema.sql
var ddl string
var db *sql.DB
var queries *database.Queries

func main() {
	if err := initDatabase(); err != nil {
		log.Fatal("Failed to initialize database:", err)
	}
	defer closeDatabase()

	http.HandleFunc("GET /", homePage)
	http.HandleFunc("GET /x/view/{id}", viewPartial)
	http.HandleFunc("GET /x/upload", uploadPartial)

	http.HandleFunc("GET /device/rtc", rtc)
	http.HandleFunc("GET /device/alarm", alarm)

	// http.HandleFunc("GET /image/{id}", getImage)
	http.HandleFunc("POST /image", createImage)
	// http.HandleFunc("DELETE /image/{id}", deleteImage)
	// http.HandleFunc("PUT /image/next/{id}", putNextImage)

	slog.Info("Starting Inkarta server...")
	log.Fatal(http.ListenAndServe(":5000", nil))
}

/* Database */

func initDatabase() error {
	var err error

	slog.Info("Initializing database...")

	// Open database
	db, err = sql.Open("sqlite", dsnURI)
	if err != nil {
		return err
	}

	// Create table
	ctx := context.Background()
	if _, err = db.ExecContext(ctx, ddl); err != nil {
		return err
	}

	queries = database.New(db)

	return err
}

func closeDatabase() {
	if err := db.Close(); err != nil {
		slog.Error("Failed to close database", "error", err)
	}
}

/* UI */

func homePage(w http.ResponseWriter, _ *http.Request) {
	ctx := context.Background()
	images, err := queries.ListImages(ctx)
	if err != nil {
		slog.Error("Failed to fetch list of images", "error", err)
		w.WriteHeader(http.StatusInternalServerError)
		return
	}

	tmpl := template.Must(template.ParseFiles("templates/index.html"))
	tmpl.Execute(w, images)
}

func viewPartial(w http.ResponseWriter, r *http.Request) {
	idValue := r.PathValue("id")
	id, err := strconv.ParseInt(idValue, 10, 64)
	if err != nil {
		slog.Error("Failed to parse image id", "error", err)
		w.WriteHeader(http.StatusBadRequest)
		return
	}

	ctx := context.Background()
	image, err := queries.GetImage(ctx, id)
	if err != nil {
		slog.Error("Failed to retrieve image from database", "error", err)
		http.NotFound(w, r)
		return
	}

	data := map[string]any{
		"Image": image,
	}

	tmpl := template.Must(template.ParseFiles("templates/view.html"))
	tmpl.Execute(w, data)
}

func uploadPartial(w http.ResponseWriter, _ *http.Request) {
	tmpl := template.Must(template.ParseFiles("templates/upload.html"))
	tmpl.Execute(w, nil)
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

func getImage(w http.ResponseWriter, r *http.Request) {
	// // Parse optional parameters for image size

	// var width, height *int

	// if widthValue := r.Form.Get("width"); widthValue != "" {
	// 	if intValue, err := strconv.Atoi(widthValue); err == nil {
	// 		width = &intValue
	// 	}
	// }

	// if heightValue := r.Form.Get("height"); heightValue != "" {
	// 	if intValue, err := strconv.Atoi(heightValue); err == nil {
	// 		height = &intValue
	// 	}
	// }

	ctx := context.Background()
	image, err := queries.GetRandomImage(ctx)
	if err != nil {
		slog.Error("Failed to get image from database", "error", err)
		w.WriteHeader(http.StatusInternalServerError)
		return
	}

	w.Header().Set("Content-Type", "image/bmp")
	w.Header().Set("Content-Length", strconv.Itoa(len(image.Data)))

	if _, err := w.Write(image.Data); err != nil {
		slog.Error("Failed to send image", "error", err)
		w.WriteHeader(http.StatusInternalServerError)
		return
	}

	slog.Info("Returning image at full resolution", "title", image.Title)
}

func createImage(w http.ResponseWriter, r *http.Request) {
	// Extract form values
	title := r.FormValue("title")
	artist := r.FormValue("artist")
	dark := r.FormValue("dark") == "on"

	file, _, err := r.FormFile("image")
	defer file.Close()

	if err != nil {
		slog.Error("Failed to read uploaded image file", "error", err)
		w.WriteHeader(http.StatusBadRequest)
		return
	}

	// Process image into bitmap for Inkplate
	bitmap, err := processImage(file)
	if err != nil {
		slog.Error("Failed to process image into bitmap", "error", err)
	}

	// Determine background color
	var background int64
	if dark {
		background = 0
	} else {
		background = 255
	}

	// Store image into database
	ctx := context.Background()
	params := database.CreateImageParams{
		Title:      title,
		Artist:     artist,
		Background: background,
		Data:       bitmap,
	}

	if err := queries.CreateImage(ctx, params); err != nil {
		slog.Error("Failed to create image", "error", err)
		w.WriteHeader(http.StatusInternalServerError)
		return
	}

	slog.Info("Created new image", "title", title, "artist", artist)

	w.Header().Set("HX-Refresh", "true")
	w.WriteHeader(http.StatusOK)
}

func processImage(f multipart.File) ([]byte, error) {
	src, _, err := image.Decode(f)
	if err != nil {
		return nil, err
	}

	bounds := src.Bounds()
	dst := image.NewGray(bounds)
	draw.FloydSteinberg.Draw(dst, bounds, src, image.Point{})

	var buffer bytes.Buffer
	if err := bmp.Encode(&buffer, dst); err != nil {
		return nil, err
	}

	return buffer.Bytes(), nil
}
