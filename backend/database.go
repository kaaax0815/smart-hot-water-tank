package backend

import (
	"context"
	"database/sql"
	"fmt"
	"log"
	"os"

	"github.com/golang-migrate/migrate/v4"
	migratePgx "github.com/golang-migrate/migrate/v4/database/pgx/v5"
	migrateFs "github.com/golang-migrate/migrate/v4/source/iofs"
	"github.com/jackc/pgx/v5"
	"github.com/kaaax0815/smart-hot-water-tank/backend/database"
	sqlc "github.com/kaaax0815/smart-hot-water-tank/backend/database/pkg"
)

func migrateDb() {
	fs := database.GetMigrationsFS()

	db, err := sql.Open("pgx/v5", getConnDetails())
	if err != nil {
		log.Fatalf("Failed to open database connection for migration: %v", err)
	}
	defer db.Close()

	driver, err := migratePgx.WithInstance(db, &migratePgx.Config{})
	if err != nil {
		log.Fatalf("Failed to create migrate driver: %v", err)
	}

	source, err := migrateFs.New(fs, "migrations")
	if err != nil {
		log.Fatalf("Failed to create migrate source: %v", err)
	}

	m, err := migrate.NewWithInstance("fs", source, "postgres", driver)
	if err != nil {
		log.Fatalf("Failed to create migrate instance: %v", err)
	}

	err = m.Up()
	if err != nil && err != migrate.ErrNoChange {
		log.Fatalf("Failed to apply migrations: %v", err)
	}

	log.Println("Database migrations applied successfully")
}

func getConnDetails() string {
	host := os.Getenv("DB_HOST")
	name := os.Getenv("DB_NAME")
	user := os.Getenv("DB_USER")
	pass := os.Getenv("DB_PASSWORD")

	if host == "" || name == "" || user == "" || pass == "" {
		log.Fatal("Database connection details are not fully set in environment variables")
	}

	return fmt.Sprintf("postgres://%s:%s@%s/%s?sslmode=disable", user, pass, host, name)
}

type Db struct {
	*sqlc.Queries
	db  *pgx.Conn
	ctx context.Context
}

var dbInstance *Db

func InitDB(ctx context.Context) *Db {
	migrateDb()
	db, err := pgx.Connect(ctx, getConnDetails())
	if err != nil {
		log.Fatalf("Failed to connect to database: %v", err)
	}

	queries := sqlc.New(db)

	dbInstance = &Db{
		Queries: queries,
		db:      db,
		ctx:     ctx,
	}

	log.Println("Database initialized successfully")

	return dbInstance
}

func GetDB() *Db {
	if dbInstance == nil {
		log.Fatal("Database not initialized. Call InitDB first.")
	}
	return dbInstance
}

func (d *Db) Close() error {
	return d.db.Close(d.ctx)
}
