package main

import (
	"context"
	"log"
	"os"
	"os/signal"
	"syscall"

	"github.com/kaaax0815/smart-hot-water-tank/backend"
)

func main() {
	ctx := context.Background()

	db := backend.InitDB(ctx)
	defer db.Close()

	if os.Getenv("SEED") == "dev" {
		err := backend.GetDB().DevSeedSensor(ctx)
		if err != nil {
			log.Fatalf("Failed to seed database: %v", err)
		}
		err = backend.GetDB().DevSeedLogin(ctx)
		if err != nil {
			log.Fatalf("Failed to seed database: %v", err)
		}
	}

	srv := backend.StartServer("0.0.0.0", "8080")
	go srv.Start()
	defer srv.Stop(ctx)

	sigChan := make(chan os.Signal, 1)
	signal.Notify(sigChan, os.Interrupt, syscall.SIGINT, syscall.SIGTERM)

	<-sigChan
	log.Println("Received shutdown signal. Shutting down gracefully...")
}
