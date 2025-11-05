# Test Scripts

This directory contains automated testing and verification scripts for the Handy project.

## Scripts

### `run_all_tests.sh`

Comprehensive test suite runner that executes all tests across all development phases.

**Usage:**
```bash
./scripts/run_all_tests.sh
```

**What it does:**
- Runs all Rust unit tests
- Runs integration tests
- Checks code formatting and linting
- Runs frontend tests
- Generates coverage reports (if cargo-tarpaulin is installed)
- Creates detailed test report in `test_results/`

**Requirements:**
- Rust toolchain
- Bun (for frontend tests)
- Optional: cargo-tarpaulin (for coverage)
- Optional: Ollama (for Ollama integration tests)

**Output:**
- Console output with color-coded results
- Test report in `test_results/test_report_TIMESTAMP.txt`
- Individual test logs in `test_results/`

### `verify_logging.sh`

Verifies that all `println!` and `eprintln!` statements have been replaced with proper logging macros.

**Usage:**
```bash
./scripts/verify_logging.sh
```

**What it checks:**
- `println!` statements in source files
- `eprintln!` statements in source files
- `print!` statements in source files

**Exit codes:**
- 0: No violations found (all logging uses log crate)
- 1: Violations found (provides locations)

## Installation

### Prerequisites

1. **Rust toolchain** (required):
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **Bun** (required for frontend tests):
   ```bash
   curl -fsSL https://bun.sh/install | bash
   ```

3. **cargo-tarpaulin** (optional, for coverage):
   ```bash
   cargo install cargo-tarpaulin
   ```

4. **Ollama** (optional, for Ollama integration tests):
   ```bash
   # macOS/Linux
   curl -fsSL https://ollama.com/install.sh | sh

   # Then pull a test model
   ollama pull llama3.2:3b
   ```

## Quick Start

Run all tests:
```bash
./scripts/run_all_tests.sh
```

Check logging compliance:
```bash
./scripts/verify_logging.sh
```

Run specific test categories:
```bash
# Only unit tests
cd src-tauri && cargo test --lib

# Only integration tests
cd src-tauri && cargo test --test integration

# Only Ollama tests (requires Ollama running)
cd src-tauri && cargo test --test ollama_integration -- --ignored

# Only frontend tests
bun test
```

## CI Integration

These scripts can be integrated into CI/CD pipelines:

### GitHub Actions Example

```yaml
name: Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable

      - name: Install Bun
        uses: oven-sh/setup-bun@v1

      - name: Run Tests
        run: ./scripts/run_all_tests.sh

      - name: Upload Test Results
        if: always()
        uses: actions/upload-artifact@v3
        with:
          name: test-results
          path: test_results/
```

## Troubleshooting

### Tests fail with "command not found"

**Solution:** Install missing prerequisites (see Installation section above)

### Ollama tests always skip

**Solution:**
1. Install Ollama: `curl -fsSL https://ollama.com/install.sh | sh`
2. Start Ollama: `ollama serve` (in a separate terminal)
3. Pull a model: `ollama pull llama3.2:3b`
4. Run tests again

### Permission denied errors

**Solution:** Make scripts executable:
```bash
chmod +x scripts/*.sh
```

### Coverage report not generated

**Solution:** Install cargo-tarpaulin:
```bash
cargo install cargo-tarpaulin
```

## Development Workflow

1. **Before committing:**
   ```bash
   ./scripts/verify_logging.sh
   cd src-tauri && cargo fmt
   cd src-tauri && cargo clippy
   ```

2. **Before creating PR:**
   ```bash
   ./scripts/run_all_tests.sh
   ```

3. **Continuous testing during development:**
   ```bash
   cd src-tauri && cargo watch -x test
   ```

## Test Reports

Test reports are saved in `test_results/` with timestamps:

```
test_results/
├── test_report_20250105_143022.txt   # Overall report
├── Build_Debug.log                    # Individual test logs
├── All_Unit_Tests.log
├── Integration_Tests.log
└── coverage/                          # Coverage HTML reports
    └── index.html
```

## Adding New Tests

To add tests to the suite:

1. **Add Rust test:**
   - Create test in `src-tauri/tests/`
   - Update `run_all_tests.sh` to include it

2. **Add Frontend test:**
   - Add test to frontend test files
   - Tests automatically included in `bun test`

3. **Add benchmark:**
   - Create benchmark in `src-tauri/benches/`
   - Automatically included in `cargo bench`

## Support

For issues with tests or scripts:
- Check [IMPLEMENTATION_GUIDE.md](../IMPLEMENTATION_GUIDE.md)
- Check [DETAILED_IMPLEMENTATION_ROADMAP.md](../DETAILED_IMPLEMENTATION_ROADMAP.md)
- Open an issue on GitHub

---

*Last updated: 2025-11-05*
