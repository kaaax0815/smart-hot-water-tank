-- name: GetSensorByApiKey :one
SELECT id, api_key FROM sensors WHERE api_key = $1 LIMIT 1;

-- name: DevSeedSensor :exec
INSERT INTO sensors (name, location, unit, api_key)
VALUES ('Test Sensor', 'Test Location', '°C', '9c2320fa97aa6a17b35e01db9bcc0b251898dc5a95178bbb0ee1dde4dc4a7accba8f7746573a6869f68554f07d2ac2885d4b4b0afe9f544d474f31b89ff6ccbb');

-- name: GetAllSensors :many
SELECT
    s.id,
    s.name,
    s.location,
    s.unit,
    -- Create JSON object of the latest measurement
    json_build_object(
        'temperature', m.temperature,
        'cpu', m.cpu,
        'time', m.time
    ) AS latest_measurement
FROM sensors s
LEFT JOIN LATERAL (
    SELECT *
    FROM measurements m
    WHERE m.sensor_id = s.id
    ORDER BY m.time DESC
    LIMIT 1
) m ON true;