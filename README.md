# ulmensa

A command-line utility for retrieving and displaying current meal plans from the University of Ulm canteen.

## Features

- Fetch current and upcoming meal plans
- Display nutritional information for meals
- Multiple output formats (plain text and JSON)
- Configurable date range for meal planning

## Installation

```bash
cargo install ulmensa
```

## Usage

```bash
ulmensa [OPTIONS]
```

### Options

- `-n, --nutritional-info`: Display additional nutritional values for each meal
- `-d, --days <DAYS>`: Show meal plans for specified number of days ahead (default: 0)
- `-j, --json`: Output the meal plan in JSON format
- `-h, --help`: Display help information
- `-V, --version`: Show version information

### Examples

Display today's meal plan:
```bash
ulmensa
```

Show meals with nutritional information:
```bash
ulmensa --nutritional-info
```

Get meal plans for the next 5 days in JSON format:
```bash
ulmensa --days 5 --json
```

## Output Formats

### Plain Text
By default, the program outputs meal plans in a human-readable text format.

### JSON
When using the `--json` flag, the output will be formatted as JSON, making it suitable for parsing and integration with other tools.

## Building from Source

1. Clone the repository
2. Run `cargo build --release`
3. The binary will be available in `target/release/ulmensa`

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the GPL License - see the LICENSE file for details.
