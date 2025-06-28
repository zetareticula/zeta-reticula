package dsl

import (
	"encoding/json"
	"io/ioutil"
)

// Package dsl provides functionality to load and manage a schema for vector data
// and quantized layers. It supports default values for vectors and loosely coupled references
// between vectors and layers. The schema can be loaded from a JSON file, which allows for
// flexible configuration of vector dimensions, default data, and quantization parameters.
// The schema is designed to be extensible, allowing for future additions of new vector types
// and quantization methods without breaking existing functionality.

type SchemaDefaults struct {
	VectorData []float32 `json:"vector_data,omitempty"` // Default data for vectors
	LayerRef   string    `json:"layer_ref,omitempty"`   // Default reference for layers
}

// Vector represents a vector in the schema with its ID, dimensions, data, and optional layer reference.
// It includes a default data field that can be preloaded with default values.

type Vector struct {
	ID          string    `json:"id"`
	Dimensions  int       `json:"dimensions"`
	Data        []float32 `json:"data,omitempty"`
	LayerRef    string    `json:"layer_ref,omitempty"`
	DefaultData []float32 `json:"default_data"` // Preloaded default
}

type QuantizedLayer struct {
	ID       string   `json:"id"`
	BitDepth int      `json:"bit_depth"`
	Vectors  []string `json:"vectors,omitempty"` // Loosely coupled references
}

type Schema struct {
	Vectors  []Vector         `json:"vectors"`
	Layers   []QuantizedLayer `json:"layers"`
	Defaults map[string]any   `json:"defaults"`
}

func LoadSchema(filePath string) (*Schema, error) {
	data, err := ioutil.ReadFile(filePath)
	if err != nil {
		return nil, err
	}

	var schema Schema
	if err := json.Unmarshal(data, &schema); err != nil {
		return nil, err
	}

	// Apply defaults
	for i, v := range schema.Vectors {
		if len(v.DefaultData) == 0 && schema.Defaults["vector_data"] != nil {
			schema.Vectors[i].DefaultData = schema.Defaults["vector_data"].([]float32)
		}
	}

	return &schema, nil
}

// SaveSchema saves the schema to a JSON file.
func SaveSchema(filePath string, schema *Schema) error {
	data, err := json.MarshalIndent(schema, "", "  ")
	if err != nil {
		return err
	}

	var defaults = make(map[string]any)
	if len(schema.Vectors) > 0 {
		defaults["vector_data"] = schema.Vectors[0].DefaultData
	}

	schema.Defaults = defaults

	return ioutil.WriteFile(filePath, data, 0644)
}
