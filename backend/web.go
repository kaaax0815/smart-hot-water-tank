package backend

import (
	"embed"
	"fmt"
	"io/fs"
	"net/http"
)

//go:embed web/dist/*
var webFS embed.FS

func CreateFrontendHandler() http.HandlerFunc {
	subFS, err := fs.Sub(webFS, "web/dist")
	if err != nil {
		panic(fmt.Errorf("failed getting the sub tree for the site files: %w", err))
	}

	indexFS := indexFS{
		data: subFS,
	}

	return func(w http.ResponseWriter, r *http.Request) {
		http.FileServer(http.FS(indexFS)).ServeHTTP(w, r)
	}
}

type indexFS struct {
	data fs.FS
}

func (o indexFS) Open(name string) (fs.File, error) {
	if name == "index.html" {
		return o.data.Open("index.html")
	}
	f, err := o.data.Open(name)
	if err != nil {
		return o.data.Open("index.html")
	}
	return f, nil
}
