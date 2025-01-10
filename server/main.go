package main

import (
	"fmt"
	"html/template"
	"log"
	"log/slog"
	"net/http"
	"time"
)

func main() {
	http.HandleFunc("GET /", home)
	// http.HandleFunc("GET /upload", home)
	// http.HandleFunc("GET /partial/image/{id}", home)

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

/* Routes */

func home(w http.ResponseWriter, _ *http.Request) {
	tmpl := template.Must(template.ParseFiles("templates/base.html", "templates/index.html"))
	data := map[string]string{
		"Title": "Gallery",
	}

	tmpl.Execute(w, data)
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
