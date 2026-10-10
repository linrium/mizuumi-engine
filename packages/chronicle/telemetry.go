package main

import (
	"context"
	"fmt"
	"os"
	"strconv"
	"strings"
	"time"

	"github.com/gofiber/fiber/v3"
	"go.opentelemetry.io/otel"
	"go.opentelemetry.io/otel/attribute"
	"go.opentelemetry.io/otel/codes"
	"go.opentelemetry.io/otel/exporters/otlp/otlptrace/otlptracehttp"
	"go.opentelemetry.io/otel/propagation"
	"go.opentelemetry.io/otel/sdk/resource"
	sdktrace "go.opentelemetry.io/otel/sdk/trace"
	semconv "go.opentelemetry.io/otel/semconv/v1.26.0"
	"go.opentelemetry.io/otel/trace"
)

type telemetryShutdown func(context.Context) error

func setupTelemetry(ctx context.Context) (telemetryShutdown, error) {
	if !getenvBool("CHRONICLE_OTEL_ENABLED", false) &&
		os.Getenv("OTEL_EXPORTER_OTLP_ENDPOINT") == "" &&
		os.Getenv("OTEL_EXPORTER_OTLP_TRACES_ENDPOINT") == "" {
		return func(context.Context) error { return nil }, nil
	}

	serviceName := getenv("OTEL_SERVICE_NAME", "chronicle")
	attrs := []attribute.KeyValue{
		semconv.ServiceName(serviceName),
	}
	if version := os.Getenv("OTEL_SERVICE_VERSION"); version != "" {
		attrs = append(attrs, semconv.ServiceVersion(version))
	}
	res, err := resource.New(ctx,
		resource.WithFromEnv(),
		resource.WithTelemetrySDK(),
		resource.WithProcess(),
		resource.WithAttributes(attrs...),
	)
	if err != nil {
		return nil, fmt.Errorf("build OTel resource: %w", err)
	}

	opts := []otlptracehttp.Option{}
	if endpoint := os.Getenv("OTEL_EXPORTER_OTLP_TRACES_ENDPOINT"); endpoint != "" {
		opts = append(opts, otlptracehttp.WithEndpointURL(endpoint))
	} else if endpoint := os.Getenv("OTEL_EXPORTER_OTLP_ENDPOINT"); endpoint != "" {
		opts = append(opts, otlptracehttp.WithEndpointURL(strings.TrimSuffix(endpoint, "/")+"/v1/traces"))
	}
	if insecure, _ := strconv.ParseBool(os.Getenv("OTEL_EXPORTER_OTLP_INSECURE")); insecure {
		opts = append(opts, otlptracehttp.WithInsecure())
	}

	exporter, err := otlptracehttp.New(ctx, opts...)
	if err != nil {
		return nil, fmt.Errorf("build OTLP trace exporter: %w", err)
	}

	tp := sdktrace.NewTracerProvider(
		sdktrace.WithBatcher(exporter, sdktrace.WithBatchTimeout(5*time.Second)),
		sdktrace.WithResource(res),
	)
	otel.SetTracerProvider(tp)
	otel.SetTextMapPropagator(propagation.NewCompositeTextMapPropagator(
		propagation.TraceContext{}, propagation.Baggage{},
	))

	return tp.Shutdown, nil
}

func tracingMiddleware(serviceName string) fiber.Handler {
	tracer := otel.Tracer(serviceName)
	propagator := otel.GetTextMapPropagator()
	return func(c fiber.Ctx) error {
		carrier := propagation.HeaderCarrier{}
		c.Request().Header.VisitAll(func(k, v []byte) {
			carrier.Set(string(k), string(v))
		})
		ctx := propagator.Extract(c.Context(), carrier)

		method := string(c.Request().Header.Method())
		route := c.Route().Path
		if route == "" {
			route = string(c.Request().URI().Path())
		}
		spanName := method + " " + route

		ctx, span := tracer.Start(ctx, spanName,
			trace.WithSpanKind(trace.SpanKindServer),
			trace.WithAttributes(
				semconv.HTTPRequestMethodKey.String(method),
				semconv.URLPath(string(c.Request().URI().Path())),
				semconv.ServerAddress(string(c.Request().URI().Host())),
				semconv.UserAgentOriginal(string(c.Request().Header.UserAgent())),
			),
		)
		defer span.End()

		c.SetContext(ctx)
		err := c.Next()

		status := c.Response().StatusCode()
		span.SetAttributes(semconv.HTTPResponseStatusCode(status))
		if err != nil {
			span.RecordError(err)
			span.SetStatus(codes.Error, err.Error())
		} else if status >= 500 {
			span.SetStatus(codes.Error, fmt.Sprintf("http %d", status))
		}
		return err
	}
}
