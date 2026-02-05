package backend

import (
	"context"
	"encoding/json"
	"log"
	"net/http"
	"strings"
	"time"

	"github.com/go-chi/chi/v5"
	"github.com/go-playground/validator/v10"
	"github.com/jackc/pgx/v5/pgtype"
	sqlc "github.com/kaaax0815/smart-hot-water-tank/backend/database/pkg"
)

func getBearerToken(r *http.Request) (string, bool) {
	auth := r.Header.Get("Authorization")
	if auth == "" || !strings.HasPrefix(auth, "Bearer ") {
		return "", false
	}
	token := strings.TrimPrefix(auth, "Bearer ")
	return token, true
}

func getSensorFromContext(ctx context.Context) (*sqlc.GetSensorByApiKeyRow, bool) {
	sensor, ok := ctx.Value("sensor").(*sqlc.GetSensorByApiKeyRow)
	return sensor, ok
}

func sensorAuthMiddleware(next http.Handler) http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		token, ok := getBearerToken(r)
		if !ok {
			http.Error(w, "Unauthorized", http.StatusUnauthorized)
			return
		}
		sensor, err := GetDB().GetSensorByApiKey(r.Context(), token)
		if err != nil {
			http.Error(w, "Unauthorized", http.StatusUnauthorized)
			return
		}
		r = r.WithContext(context.WithValue(r.Context(), "sensor", &sensor))
		next.ServeHTTP(w, r)
	})
}

func getLoginFromContext(ctx context.Context) (*sqlc.GetLoginByApiKeyRow, bool) {
	login, ok := ctx.Value("login").(*sqlc.GetLoginByApiKeyRow)
	return login, ok
}

func loginAuthMiddleware(next http.Handler) http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		token, ok := getBearerToken(r)
		if !ok {
			http.Error(w, "Unauthorized", http.StatusUnauthorized)
			return
		}
		login, err := GetDB().GetLoginByApiKey(r.Context(), token)
		if err != nil {
			http.Error(w, "Unauthorized", http.StatusUnauthorized)
			return
		}
		r = r.WithContext(context.WithValue(r.Context(), "login", &login))
		next.ServeHTTP(w, r)
	})
}

func CreateApiRouter() *chi.Mux {
	r := chi.NewRouter()

	// ingest group
	r.Group(func(r chi.Router) {
		r.Use(sensorAuthMiddleware)

		r.Post("/ingest", postIngest)
	})

	// web group
	r.Group(func(r chi.Router) {
		r.Use(loginAuthMiddleware)

		r.Get("/sensors", getSensors)
	})

	return r
}

type IngestRequest struct {
	Temperature float64  `json:"temp" validate:"required"`
	Cpu         *float64 `json:"cpu,omitempty"`
}

type IngestResponse struct {
	// sleep time in milliseconds
	SleepTime int64  `json:"sleep_time"`
	Message   string `json:"message"`
}

func postIngest(w http.ResponseWriter, r *http.Request) {
	req := new(IngestRequest)

	decoder := json.NewDecoder(r.Body)
	decoder.DisallowUnknownFields()
	err := decoder.Decode(req)
	if err != nil {
		log.Printf("Error decoding request body: %v", err)
		http.Error(w, err.Error(), http.StatusBadRequest)
		return
	}

	validate := validator.New()
	err = validate.Struct(req)
	if err != nil {
		log.Printf("Validation error: %v", err)
		http.Error(w, err.Error(), http.StatusBadRequest)
		return
	}

	sensor, ok := getSensorFromContext(r.Context())
	if !ok {
		log.Printf("Sensor not found in context")
		http.Error(w, "Internal Server Error", http.StatusInternalServerError)
		return
	}

	dbErr := GetDB().InsertMeasurement(r.Context(), sqlc.InsertMeasurementParams{
		SensorID:    sensor.ID,
		Temperature: req.Temperature,
		Cpu:         req.Cpu,
		Time:        pgtype.Timestamptz{Time: time.Now(), Valid: true},
	})
	if dbErr != nil {
		log.Printf("Error inserting measurement: %v", dbErr)
		http.Error(w, dbErr.Error(), http.StatusInternalServerError)
		return
	}

	resp := IngestResponse{
		SleepTime: (10 * time.Minute).Milliseconds(),
		Message:   "Measurement ingested successfully",
	}

	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(resp)

	w.WriteHeader(http.StatusOK)
}

type SensorResponse struct {
	ID                int32           `json:"id"`
	Name              string          `json:"name"`
	Location          string          `json:"location"`
	Unit              string          `json:"unit"`
	LatestMeasurement json.RawMessage `json:"latest_measurement,omitempty"`
}

func getSensors(w http.ResponseWriter, r *http.Request) {
	_, ok := getLoginFromContext(r.Context())
	if !ok {
		log.Printf("Login not found in context")
		http.Error(w, "Internal Server Error", http.StatusInternalServerError)
		return
	}

	sensors, err := GetDB().GetAllSensors(r.Context())
	if err != nil {
		log.Printf("Error fetching sensors: %v", err)
		http.Error(w, err.Error(), http.StatusInternalServerError)
		return
	}

	var sensorResponses []SensorResponse
	for _, sensor := range sensors {
		sensorResponses = append(sensorResponses, SensorResponse{
			ID:                sensor.ID,
			Name:              sensor.Name,
			Location:          sensor.Location,
			Unit:              sensor.Unit,
			LatestMeasurement: sensor.LatestMeasurement,
		})
	}

	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(sensorResponses)

	w.WriteHeader(http.StatusOK)
}
