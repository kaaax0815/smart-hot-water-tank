CREATE TABLE measurements (
    time TIMESTAMPTZ NOT NULL,
    sensor_id INTEGER NOT NULL,
    temperature DOUBLE PRECISION NOT NULL,
    cpu DOUBLE PRECISION,
    FOREIGN KEY (sensor_id) REFERENCES sensors (id) ON DELETE CASCADE
)
WITH(
   tsdb.hypertable,
   tsdb.segmentby = 'sensor_id',
   tsdb.orderby = 'time DESC'
);