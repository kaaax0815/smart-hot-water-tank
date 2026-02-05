package backend

import (
	"context"
	"log"
	"net"
	"net/http"
	"time"

	"github.com/go-chi/chi/v5"
	"github.com/go-chi/chi/v5/middleware"
)

func CreateRouter() *chi.Mux {
	r := chi.NewRouter()
	r.Use(middleware.RequestID)
	r.Use(middleware.Logger)
	r.Use(middleware.Recoverer)
	r.Use(middleware.Timeout(15 * time.Second))

	r.Handle("/*", CreateFrontendHandler())
	r.Mount("/api", CreateApiRouter())

	return r
}

func StartServer(host, port string) *server {
	hostport := net.JoinHostPort(host, port)
	router := CreateRouter()

	httpServer := &http.Server{
		Addr:    hostport,
		Handler: router,
	}

	return &server{httpServer}
}

type server struct {
	*http.Server
}

func (s *server) Start() {
	log.Printf("Server running at http://%s", s.Addr)
	err := s.ListenAndServe()

	if err != nil && err != http.ErrServerClosed {
		log.Fatalf("Server failed: %v", err)
	}
}

func (s *server) Stop(ctx context.Context) {
	log.Println("Stopping server...")
	if err := s.Server.Shutdown(ctx); err != nil {
		log.Printf("Stopping server failed: %v", err)
	}
}
