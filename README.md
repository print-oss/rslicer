# RSlicer

Not to be confused with an actual slicer, RSlicer is a lightweight REST API server that estimates the weight of given 3D models based on given parameters. This was originally built as a tool for estimating cost for jobs on print.computer

## Features

- Calculate 3D model weight based on:
  - Model dimensions (X, Y, Z)
  - Infill percentage
  - Material types
- REST API interface for simple integration
- Command-line interface for quick calculations
- Current only supports STL files

## Installation

1. Ensure you have Rust installed
2. Clone this repository:
   ```bash
   git clone https://github.com/print-oss/rslicer.git
   cd rslicer
   ```
3. Build the project:
   ```bash
   cargo build --release
   ```

## Usage

### Command Line Interface

```bash
cargo run <stl-file-path> <x-dim> <y-dim> <z-dim> <infill_percentage> [material]
```

Parameters:

- `stl-file-path`: Path to the STL file
- `x-dim`: Desired X dimension in millimeters
- `y-dim`: Desired Y dimension in millimeters
- `z-dim`: Desired Z dimension in millimeters
- `infill_percentage`: Infill percentage (0-100)
- `material`: Optional material type (pla, abs, petg, tpu). Defaults to PLA if not specified.

Example:

```bash
cargo run model.stl 100 100 100 20 petg
```

### REST API Server

To start the API server:

```bash
cargo run --api
```

The server will start on `http://localhost:8080`.

#### API Endpoints

- `POST /calculate`
  - Request body: Multipart form data
    - `file`: STL file
    - `x_dim`: X dimension in millimeters
    - `y_dim`: Y dimension in millimeters
    - `z_dim`: Z dimension in millimeters
    - `infill_percentage`: Infill percentage (0-100)
    - `material`: Material type (pla, abs, petg, tpu)
  - Response: JSON with weight in grams
    ```json
    {
      "weight_grams": "123.45"
    }
    ```

## Supported Materials

- PLA (default): 1.24 g/cm³
- ABS: 1.04 g/cm³
- PETG: 1.27 g/cm³
- TPU: 1.21 g/cm³

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Contributing

Please feel free to open a PR!
