package main

import (
	"html/template"
	"log"
	"log/slog"
	"net/http"
)

func main() {
	http.HandleFunc("/", home)

	slog.Info("Starting Inkarta server...")
	log.Fatal(http.ListenAndServe(":5000", nil))
}

/* Routes */

func home(w http.ResponseWriter, r *http.Request) {
	tmpl := template.Must(template.ParseFiles("templates/base.html", "templates/index.html"))
	data := map[string]string{
		"Title": "Gallery",
	}

	tmpl.Execute(w, data)
}

func uploadPage(w http.ResponseWriter, r *http.Request) {
	// tmpl, _ := template.ParseFiles("templates/.html")
}
