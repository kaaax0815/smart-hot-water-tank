-- name: InsertMeasurement :exec
INSERT INTO measurements (sensor_id, temperature, cpu, time)
VALUES ($1, $2, $3, $4);
