package main

import (
	"bytes"
	"context"
	"fmt"
	"html/template"
	"log"
	"log/slog"
	"math"
	"mime/multipart"
	"net/http"
	"strconv"
	"time"

	"database/sql"
	_ "embed"
	"inkarta/internal/database"

	_ "modernc.org/sqlite"

	"image"
	"image/color"
	_ "image/gif"
	_ "image/jpeg"
	_ "image/png"

	"golang.org/x/image/bmp"
	"golang.org/x/image/draw"
)

const dsnURI = "file:inkarta.db?cache=shared&mode=rwc&journal_mode=WAL"

//go:embed schema.sql
var ddl string
var db *sql.DB
var queries *database.Queries

var current, next int64

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

	http.HandleFunc("GET /image/{id}", getImage)
	http.HandleFunc("GET /image/next", getNextImage)
	http.HandleFunc("POST /image", createImage)
	http.HandleFunc("DELETE /image/{id}", deleteImage)
	http.HandleFunc("PUT /image/next/{id}", setNextImage)

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

	currentTitle := "None"
	if current != 0 {
		if result, err := queries.GetImage(ctx, current); err == nil {
			currentTitle = result.Title
		}
	}

	nextTitle := "None"
	if next != 0 {
		if result, err := queries.GetImage(ctx, next); err == nil {
			nextTitle = result.Title
		}
	}

	data := map[string]any{
		"Current": currentTitle,
		"Next":    nextTitle,
		"Images":  images,
	}

	tmpl := template.Must(template.ParseFiles("templates/index.html"))
	tmpl.Execute(w, data)
}

func viewPartial(w http.ResponseWriter, r *http.Request) {
	id, err := parseId(r)
	if err != nil {
		w.WriteHeader(http.StatusBadRequest)
		return
	}

	ctx := context.Background()
	image, err := queries.GetImage(ctx, id)
	if err != nil {
		slog.Error("Failed to fetch image", "error", err)
		http.NotFound(w, r)
		return
	}

	data := map[string]any{
		"Next":  next,
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

// Send image data for specific ID.
func getImage(w http.ResponseWriter, r *http.Request) {
	id, err := parseId(r)
	if err != nil {
		w.WriteHeader(http.StatusBadRequest)
		return
	}

	ctx := context.Background()
	result, err := queries.GetImage(ctx, id)
	if err != nil {
		slog.Error("Failed to fetch image", "id", id, "error", err)
		http.NotFound(w, r)
		return
	}

	sendImage(w, r, &result)
}

// Send image data for next image.
func getNextImage(w http.ResponseWriter, r *http.Request) {
	ctx := context.Background()

	var result database.Image
	var err error

	if next == 0 {
		// Select a random image, since no next ID set
		result, err = queries.GetRandomImage(ctx)
	} else {
		// Select image based on next ID
		result, err = queries.GetImage(ctx, next)
	}

	if err != nil {
		slog.Error("Failed to fetch next image", "error", err)
		w.WriteHeader(http.StatusInternalServerError)
		return
	}

	// Update current and next states
	current = result.ID
	if result, err := queries.GetRandomImage(ctx); err == nil {
		next = result.ID
	}

	sendImage(w, r, &result)
}

func sendImage(w http.ResponseWriter, r *http.Request, result *database.Image) {
	// Parse optional resizing parameters
	widthValue := r.FormValue("width")
	newWidth, _ := strconv.Atoi(widthValue)

	heightValue := r.FormValue("height")
	newHeight, _ := strconv.Atoi(heightValue)

	// Resize image if needed
	buffer := resizeImage(*result, newWidth, newHeight)
	data := buffer.Bytes()

	// Return image response
	w.Header().Set("Content-Type", "image/bmp")
	w.Header().Set("Content-Length", strconv.Itoa(len(data)))

	if _, err := w.Write(data); err != nil {
		slog.Error("Failed to send image", "error", err)
		w.WriteHeader(http.StatusInternalServerError)
		return
	}
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

	// Store image into database
	ctx := context.Background()
	params := database.CreateImageParams{
		Title:  title,
		Artist: artist,
		Dark:   dark,
		Data:   bitmap,
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

func deleteImage(w http.ResponseWriter, r *http.Request) {
	id, err := parseId(r)
	if err != nil {
		w.WriteHeader(http.StatusBadRequest)
		return
	}

	ctx := context.Background()
	err = queries.DeleteImage(ctx, id)
	if err != nil {
		slog.Error("Failed to delete image", "error", err)
		http.NotFound(w, r)
		return
	}

	// Clear state if identifiers match the deleted image

	if current == id {
		slog.Info("Reset current image")
		current = 0
	}

	if next == id {
		slog.Info("Reset next image")
		next = 0
	}

	slog.Info("Deleted image", "id", id)
	w.Header().Set("HX-Refresh", "true")
	w.WriteHeader(http.StatusOK)
}

func setNextImage(w http.ResponseWriter, r *http.Request) {
	id, err := strconv.ParseInt(r.PathValue("id"), 10, 64)
	if err != nil {
		slog.Error("Failed to parse image id", "error", err)
		w.WriteHeader(http.StatusBadRequest)
		return
	}

	next = id

	slog.Info("Selected next image", "id", id)
	w.Header().Set("HX-Refresh", "true")
	w.WriteHeader(http.StatusOK)
}

/* Image Processing */

// Converts an image into a grayscale bitmap.
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

// Resizes image to desired dimensions.
func resizeImage(result database.Image, newWidth int, newHeight int) *bytes.Buffer {
	reader := bytes.NewReader(result.Data)
	src, _, _ := image.Decode(reader)

	oldWidth := src.Bounds().Dx()
	oldHeight := src.Bounds().Dy()

	if newWidth == 0 {
		newWidth = oldWidth
	}

	if newHeight == 0 {
		newHeight = oldHeight
	}

	var buffer bytes.Buffer
	if newWidth != oldWidth || newHeight != oldHeight {
		slog.Info("Resizing image", "width", newWidth, "height", newHeight)

		// Determine fill color
		var fill color.Color
		if result.Dark {
			fill = color.Black
		} else {
			fill = color.White
		}

		// Create destination canvas with background fill
		dst := image.NewGray(image.Rect(0, 0, newWidth, newHeight))
		draw.Draw(dst, dst.Bounds(), &image.Uniform{fill}, image.Point{}, draw.Src)

		// Calculate the scaling factor to maintain aspect ratio
		scaleX := float64(newWidth) / float64(oldWidth)
		scaleY := float64(newHeight) / float64(oldHeight)
		scale := math.Min(scaleX, scaleY)

		// Calculate scaled image dimensions
		scaledWidth := int(float64(oldWidth) * scale)
		scaledHeight := int(float64(oldHeight) * scale)

		// Calculate offsets to center the scaled image
		offsetX := (newWidth - scaledWidth) / 2
		offsetY := (newHeight - scaledHeight) / 2

		// Define the rectangle for the scaled image's position
		scaledRect := image.Rect(offsetX, offsetY, offsetX+scaledWidth, offsetY+scaledHeight)

		// Scale the source image into the destination
		draw.ApproxBiLinear.Scale(dst, scaledRect, src, src.Bounds(), draw.Over, nil)
		bmp.Encode(&buffer, dst)
	} else {
		slog.Info("Returning image at full resolution", "title", result.Title)
		bmp.Encode(&buffer, src)
	}

	return &buffer
}

/* Utilities */

func parseId(r *http.Request) (int64, error) {
	id, err := strconv.ParseInt(r.PathValue("id"), 10, 64)
	if err != nil {
		slog.Error("Failed to parse image id", "error", err)
		return 0, err
	}

	return id, nil
}
