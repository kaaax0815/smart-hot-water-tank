package database

import "embed"

//go:embed migrations/*.sql
var fs embed.FS

func GetMigrationsFS() *embed.FS {
	return &fs
}
