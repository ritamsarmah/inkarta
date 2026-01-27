package main

import (
	"bytes"
	"context"
	"database/sql"
	"embed"
	"fmt"
	"html/template"
	"image"
	"image/color"
	"log"
	"mime/multipart"
	"net/http"
	"os"
	"strconv"
	"time"

	_ "image/gif"
	_ "image/jpeg"
	_ "image/png"

	"golang.org/x/image/bmp"
	"golang.org/x/image/draw"
	"ritam.me/inkarta/internal/db"

	_ "modernc.org/sqlite"
)

/* Globals */

//go:embed templates/*.tmpl
var TemplateFS embed.FS
var Templates = template.Must(template.ParseFS(TemplateFS, "templates/*.tmpl"))

var (
	CurrentId int64
	NextId    int64
)

var (
	//go:embed schema.sql
	DDL     string
	DB      *sql.DB
	Queries *db.Queries
)

/* Initialization */

func main() {
	var err error

	// Database

	dsn := env("SQLITE_DSN")
	DB, err = sql.Open("sqlite", dsn)
	if err != nil {
		log.Panic("failed to open database:", err)
	}

	ctx := context.Background()
	if _, err := DB.ExecContext(ctx, DDL); err != nil {
		log.Panic("failed to initialize database:", err)
	}
	defer DB.Close()

	Queries = db.New(DB)

	// Router

	mux := http.NewServeMux()

	mux.HandleFunc("GET /", homePage)
	mux.HandleFunc("GET /ui/upload", uploadModal)
	mux.HandleFunc("GET /ui/view/{id}", viewModal)

	mux.HandleFunc("GET /device/rtc", deviceRtc)
	mux.HandleFunc("GET /device/alarm", deviceAlarm)

	mux.HandleFunc("GET /image/{id}", getImage)
	mux.HandleFunc("GET /image/next", getNextImage)
	mux.HandleFunc("PUT /image/next/{id}", setNextImage)
	mux.HandleFunc("POST /image", createImage)
	mux.HandleFunc("DELETE /image/{id}", deleteImage)

	// Server

	host, port := env("HOST"), env("PORT")
	addr := fmt.Sprintf("%v:%v", host, port)

	log.Println("listening on", addr)

	if err := http.ListenAndServe(addr, mux); err != nil {
		log.Panic("failed to start server:", err)
	}
}

/* Routes */

func homePage(w http.ResponseWriter, r *http.Request) {
	ctx := r.Context()

	images, err := Queries.ListImageInfo(ctx)
	if err != nil {
		log.Println("failed to fetch list of images:", err)
		w.WriteHeader(http.StatusInternalServerError)
		return
	}

	currentTitle := "None"
	if CurrentId != 0 {
		if img, err := Queries.GetImage(ctx, CurrentId); err == nil {
			currentTitle = img.Title
		}
	}

	nextTitle := "None"
	if NextId != 0 {
		if img, err := Queries.GetImage(ctx, NextId); err == nil {
			nextTitle = img.Title
		}
	}

	data := map[string]any{
		"CurrentTitle": currentTitle,
		"NextTitle":    nextTitle,
		"Images":       images,
	}

	Templates.ExecuteTemplate(w, "index.tmpl", data)
}

func uploadModal(w http.ResponseWriter, r *http.Request) {
	Templates.ExecuteTemplate(w, "upload.tmpl", nil)
}

func viewModal(w http.ResponseWriter, r *http.Request) {
	id, err := parseId(r)
	if err != nil {
		w.WriteHeader(http.StatusBadRequest)
		return
	}

	ctx := r.Context()
	img, err := Queries.GetImage(ctx, id)
	if err != nil {
		log.Print("failed to fetch image:", err)
		http.NotFound(w, r)
		return
	}

	data := map[string]any{
		"Next":  NextId,
		"Image": img,
	}

	Templates.ExecuteTemplate(w, "view.tmpl", data)
}

// Returns Unix epoch timestamp for device RTC
func deviceRtc(w http.ResponseWriter, _ *http.Request) {
	timestamp := time.Now().Unix()
	log.Print("returning timestamp for real-time clock:", timestamp)
	fmt.Fprint(w, timestamp)
}

// Returns Unix epoch timestamp for next display refresh (i.e., at midnight in server's timezone)
func deviceAlarm(w http.ResponseWriter, _ *http.Request) {
	tomorrow := time.Now().AddDate(0, 0, 1)
	midnight := time.Date(tomorrow.Year(), tomorrow.Month(), tomorrow.Day(), 0, 0, 0, 0, tomorrow.Location())
	timestamp := midnight.Unix()

	fmt.Fprint(w, timestamp)
}

func getImage(w http.ResponseWriter, r *http.Request) {
	id, err := parseId(r)
	if err != nil {
		w.WriteHeader(http.StatusBadRequest)
		return
	}

	ctx := r.Context()
	img, err := Queries.GetImage(ctx, id)
	if err != nil {
		log.Printf("failed to get image %v: %v\n", id, err)
		http.NotFound(w, r)
		return
	}

	sendImage(w, r, &img)
}

func getNextImage(w http.ResponseWriter, r *http.Request) {
	ctx := r.Context()

	var img db.Image
	var err error

	if NextId != 0 {
		img, err = Queries.GetImage(ctx, NextId)
	} else {
		img, err = Queries.GetRandomImage(ctx)
	}

	if err != nil {
		log.Println("failed to get next image:", err)
		w.WriteHeader(http.StatusInternalServerError)
		return
	}

	// Update current and next states
	CurrentId = img.ID
	if img, err := Queries.GetRandomImage(ctx); err == nil {
		NextId = img.ID
	}

	sendImage(w, r, &img)
}

func setNextImage(w http.ResponseWriter, r *http.Request) {
	id, err := parseId(r)
	if err != nil {
		w.WriteHeader(http.StatusBadRequest)
		return
	}

	NextId = id

	log.Println("set next image:", id)
	w.Header().Set("HX-Refresh", "true")
	w.WriteHeader(http.StatusOK)
}

func createImage(w http.ResponseWriter, r *http.Request) {
	// Extract form values
	title := r.FormValue("title")
	artist := r.FormValue("artist")
	dark := r.FormValue("dark") == "on"

	file, _, err := r.FormFile("image")
	if err != nil {
		log.Println("failed to read uploaded image file:", err)
		w.WriteHeader(http.StatusBadRequest)
		return
	}
	defer file.Close()

	// Process image into bitmap for Inkplate
	bitmap, err := processImage(file)
	if err != nil {
		log.Println("failed to process image into bitmap:", err)
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

	if err := Queries.CreateImage(ctx, params); err != nil {
		log.Println("failed to create image:", err)
		w.WriteHeader(http.StatusInternalServerError)
		return
	}

	log.Printf("created new image: %v - %v\n", title, artist)

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
	err = Queries.DeleteImage(ctx, id)
	if err != nil {
		log.Println("failed to delete image:", err)
		http.NotFound(w, r)
		return
	}

	if CurrentId == id {
		log.Println("reset current image")
		CurrentId = 0
	}

	if NextId == id {
		log.Println("reset next image")
		NextId = 0
	}

	log.Println("deleted image", id)
	w.Header().Set("HX-Refresh", "true")
	w.WriteHeader(http.StatusOK)
}

/* Utilities */

func env(key string) string {
	value, found := os.LookupEnv(key)
	if !found {
		log.Panic("environment variable not set:", key)
	}

	return value
}

func parseId(r *http.Request) (int64, error) {
	id, err := strconv.ParseInt(r.PathValue("id"), 10, 64)
	if err != nil {
		log.Println("failed to parse image ID from request:", err)
		return 0, err
	}

	return id, nil
}

/* Image */

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

func sendImage(w http.ResponseWriter, r *http.Request, img *db.Image) {
	// Decode image data
	reader := bytes.NewReader(img.Data)
	src, _, err := image.Decode(reader)
	if err != nil {
		log.Println("failed to decode image:", err)
		w.WriteHeader(http.StatusInternalServerError)
		return
	}

	oldWidth, oldHeight := src.Bounds().Dx(), src.Bounds().Dy()
	newWidth, newHeight := oldWidth, oldHeight

	// Parse optional resizing parameters
	{
		var err error

		widthValue := r.FormValue("width")
		if widthValue != "" {
			newWidth, err = strconv.Atoi(widthValue)
			if newWidth <= 0 || err != nil {
				log.Println("invalid image width value:", widthValue)
				w.WriteHeader(http.StatusBadRequest)
				return
			}
		}

		heightValue := r.FormValue("height")
		if heightValue != "" {
			newHeight, err = strconv.Atoi(heightValue)
			if newHeight <= 0 || err != nil {
				log.Println("invalid image height value:", heightValue)
				w.WriteHeader(http.StatusBadRequest)
				return
			}
		}
	}

	var buffer bytes.Buffer

	if newWidth != oldWidth || newHeight != oldHeight {
		log.Printf("returning resized image: %v (%v x %v)\n", img.Title, newWidth, newHeight)

		// Calculate scaling factor to maintain aspect ratio
		scaleX := float32(newWidth) / float32(oldWidth)
		scaleY := float32(newHeight) / float32(oldHeight)
		scale := min(scaleX, scaleY)

		// Calculate scaled image dimensions
		scaledWidth := int(float32(oldWidth) * scale)
		scaledHeight := int(float32(oldHeight) * scale)

		// Calculate offsets to center the scaled image
		offsetX := (newWidth - scaledWidth) / 2
		offsetY := (newHeight - scaledHeight) / 2

		// Determine fill color
		var fill color.Color
		if img.Dark {
			fill = color.Black
		} else {
			fill = color.White
		}

		// Create destination canvas with background fill
		dst := image.NewGray(image.Rect(0, 0, newWidth, newHeight))
		draw.Draw(dst, dst.Bounds(), &image.Uniform{fill}, image.Point{}, draw.Src)

		// Scale the source image into the destination
		scaledRect := image.Rect(offsetX, offsetY, offsetX+scaledWidth, offsetY+scaledHeight)
		draw.ApproxBiLinear.Scale(dst, scaledRect, src, src.Bounds(), draw.Over, nil)

		err := bmp.Encode(&buffer, dst)
		if err != nil {
			log.Println("failed to encode image", err)
		}
	} else {
		log.Printf("returning image \"%v\" at full resolution\n", img.Title)

		err := bmp.Encode(&buffer, src)
		if err != nil {
			log.Println("failed to encode image", err)
		}
	}

	// Return image response
	data := buffer.Bytes()
	w.Header().Set("Content-Type", "image/bmp")
	w.Header().Set("Content-Length", strconv.Itoa(len(data)))

	if _, err := w.Write(data); err != nil {
		log.Println("failed to send image:", err)
		return
	}
}
