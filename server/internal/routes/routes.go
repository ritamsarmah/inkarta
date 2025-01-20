package routes

import (
	_ "embed"
	"fmt"
	"html/template"
	_ "image/gif"
	_ "image/jpeg"
	_ "image/png"
	"inkarta/internal/db"
	"inkarta/internal/image"
	"log/slog"
	"net/http"
	"strconv"
	"time"
)

var queries *db.Queries
var current, next int64

func NewRouter(q *db.Queries) http.Handler {
	queries = q

	mux := http.NewServeMux()

	mux.HandleFunc("GET /", homePage)
	mux.HandleFunc("GET /ui/view/{id}", viewPartial)
	mux.HandleFunc("GET /ui/upload", uploadPartial)

	mux.HandleFunc("GET /device/rtc", rtc)
	mux.HandleFunc("GET /device/alarm", alarm)

	mux.HandleFunc("GET /image/{id}", getImage)
	mux.HandleFunc("GET /image/next", getNextImage)
	mux.HandleFunc("POST /image", createImage)
	mux.HandleFunc("DELETE /image/{id}", deleteImage)
	mux.HandleFunc("PUT /image/next/{id}", setNextImage)

	return mux
}

/* UI */

func homePage(w http.ResponseWriter, r *http.Request) {
	ctx := r.Context()
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

	ctx := r.Context()
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

	ctx := r.Context()
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
	ctx := r.Context()

	var result db.Image
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

func sendImage(w http.ResponseWriter, r *http.Request, result *db.Image) {
	// Parse optional resizing parameters
	widthValue := r.FormValue("width")
	newWidth, _ := strconv.Atoi(widthValue)

	heightValue := r.FormValue("height")
	newHeight, _ := strconv.Atoi(heightValue)

	// Resize image if needed
	buffer := image.Resize(result, newWidth, newHeight)
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
	bitmap, err := image.Process(&file)
	if err != nil {
		slog.Error("Failed to process image into bitmap", "error", err)
		w.WriteHeader(http.StatusInternalServerError)
		return
	}

	// Store image into database
	ctx := r.Context()
	params := db.CreateImageParams{
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

	ctx := r.Context()
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
	id, err := parseId(r)
	if err != nil {
		w.WriteHeader(http.StatusBadRequest)
		return
	}

	next = id

	slog.Info("Set next image", "id", id)
	w.Header().Set("HX-Refresh", "true")
	w.WriteHeader(http.StatusOK)
}

/* Utilities */

func parseId(r *http.Request) (int64, error) {
	id, err := strconv.ParseInt(r.PathValue("id"), 10, 64)
	if err != nil {
		slog.Error("Failed to parse image ID from request", "error", err)
		return 0, err
	}

	return id, nil
}
