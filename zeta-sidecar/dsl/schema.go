package dsl

import (
	"encoding/json"
	"io/ioutil"
)

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
