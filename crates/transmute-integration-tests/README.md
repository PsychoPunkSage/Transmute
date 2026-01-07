# Integration Tests

## System Requirements

Tests that use `pdf_to_images()` require the pdfium library installed:

**Ubuntu/Debian:**
```bash
sudo apt install libpdfium-dev
```

**Arch:**
```bash
yay -S pdfium
```

**macOS:**
```bash
brew install pdfium
```

## Running Tests

```bash
# Run all integration tests
cargo test -p transmute-integration-tests

# Run specific test
cargo test --test integration_test -p transmute-integration-tests

# Skip PDF extraction tests (if libpdfium unavailable)
cargo test -p transmute-integration-tests test_large_batch_processing
```

## Test Coverage

- `test_end_to_end_pdf_workflow`: Images → PDF → Images round-trip (requires libpdfium)
- `test_large_batch_processing`: Concurrent processing of 100 images
- `test_memory_preservation`: Dimension preservation through PDF conversion (requires libpdfium)
