# Pullpiri Persistency Integration

## Overview

This integration replaces ETCD with the Eclipse SCORE persistency library (rust_kvs) across all Pullpiri components. The migration maintains backward compatibility with existing code while providing a centralized, efficient persistence layer.

## Architecture

### Before (ETCD-based)
```
┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│ API Server  │    │Monitor Server│   │Settings Svc │
│             │    │             │    │             │
│ etcd client │    │ etcd client │    │ etcd client │
└──────┬──────┘    └──────┬──────┘    └──────┬──────┘
       │                  │                  │
       └──────────────────┼──────────────────┘
                          │
                    ┌─────▼─────┐
                    │   ETCD    │
                    │ (External)│
                    └───────────┘
```

### After (Persistency-based)
```
┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│ API Server  │    │Monitor Server│   │Settings Svc │
│             │    │             │    │             │
│etcd compat  │    │etcd compat  │    │etcd compat  │
│  layer      │    │  layer      │    │  layer      │
└──────┬──────┘    └──────┬──────┘    └──────┬──────┘
       │                  │                  │
       └──────────────────┼──────────────────┘
                          │ gRPC
                    ┌─────▼─────┐
                    │Persistency│
                    │ Service   │
                    │(rust_kvs) │
                    └───────────┘
```

## Components Modified

### 1. Common Module (`/pullpiri/src/common`)
- **`etcd.rs`**: Replaced ETCD client with persistency service client calls
- **`persistency_client.rs`**: New gRPC client for persistency service
- **`proto/persistency.proto`**: Protocol buffer definitions for the service
- **`Cargo.toml`**: Removed `etcd-client` dependency

### 2. Persistency Service (`/pullpiri/src/server/persistency-service`)
- **`src/lib.rs`**: Service implementation wrapping rust_kvs with gRPC
- **`src/main.rs`**: Standalone service binary
- **`Cargo.toml`**: Dependencies for rust_kvs and gRPC

### 3. Compatibility Layer
The `etcd.rs` module maintains the same public API:
- `put(key, value)` → `persistency_service.set_value()`
- `get(key)` → `persistency_service.get_value()`
- `get_all_with_prefix(prefix)` → `persistency_service.get_all_with_prefix()`
- `delete(key)` → `persistency_service.remove_key()`
- `delete_all_with_prefix(prefix)` → Multiple `remove_key()` calls

## Key Benefits

### 1. Single Initialization Point
- One persistency service process for all Pullpiri components
- Shared storage eliminates data duplication
- Centralized configuration and monitoring

### 2. Improved Performance
- In-process rust_kvs operations (no network overhead for service)
- Efficient gRPC communication between components
- Better memory management with Rust

### 3. Enhanced Reliability
- Built-in data integrity checking (Adler32 checksums)
- Atomic operations within the service
- Snapshot capabilities for backup/restore

### 4. Backward Compatibility
- Existing code continues to work unchanged
- Same error handling patterns
- Identical function signatures

## Usage

### Starting the Persistency Service

```bash
# Build the service
cd /home/acrn/new_ak/score/pullpiri/src/server
cargo build -p persistency-service --release

# Run the service
./target/release/persistency-service
```

The service listens on port `47007` (configured in `common/src/lib.rs`).

### Using from Components

All existing Pullpiri components automatically use the new persistency backend through the compatibility layer:

```rust
// This code remains unchanged but now uses persistency service
use common::etcd;

async fn example() -> common::Result<()> {
    // Store data
    etcd::put("mykey", "myvalue").await?;
    
    // Retrieve data
    let value = etcd::get("mykey").await?;
    
    // Get all with prefix
    let kvs = etcd::get_all_with_prefix("prefix/").await?;
    
    Ok(())
}
```

### Testing the Integration

Use the provided test script:

```bash
cd /home/acrn/new_ak/score/pullpiri
./scripts/test_persistency_integration.sh
```

## Configuration

### Service Configuration
The persistency service uses the same host IP configuration as other Pullpiri components (from `settings.yaml`).

### Data Storage
- Data is stored in JSON format by rust_kvs
- Default location: Current working directory
- Files: `kvs_*.json` and `hash_*.json`

### Network Configuration
- **Port**: 47007 (default, configurable in `common/src/lib.rs`)
- **Protocol**: gRPC (HTTP/2)
- **Security**: Currently unencrypted (can be enhanced with TLS)

## Migration Notes

### For Developers
- No code changes required for existing ETCD usage
- New features can use enhanced rust_kvs capabilities
- Error handling remains the same

### For Deployment
- Start persistency service before other components
- Ensure port 47007 is available
- Consider running as a system service (systemd)

### Data Migration
- Existing ETCD data needs manual migration
- Use export/import scripts for data transfer
- Test thoroughly in staging environment

## Future Enhancements

1. **TLS Security**: Add mutual TLS for production deployments
2. **Clustering**: Support for distributed persistency service
3. **Monitoring**: Expose metrics and health endpoints
4. **Configuration**: External configuration file support
5. **Backup**: Automated snapshot scheduling

## Troubleshooting

### Service Won't Start
- Check if port 47007 is available: `netstat -tulpn | grep 47007`
- Verify rust_kvs dependencies are installed
- Check host IP configuration in settings.yaml

### Connection Errors
- Ensure persistency service is running
- Verify network connectivity
- Check firewall settings for port 47007

### Data Issues
- Check file permissions in working directory
- Verify JSON files are not corrupted
- Use rust_kvs diagnostic tools

## Files Modified/Created

### New Files
- `/pullpiri/src/common/proto/persistency.proto`
- `/pullpiri/src/common/src/persistency_client.rs`
- `/pullpiri/src/server/persistency-service/`
- `/pullpiri/scripts/test_persistency_integration.sh`

### Modified Files
- `/pullpiri/src/common/src/etcd.rs` (replaced implementation)
- `/pullpiri/src/common/src/lib.rs` (added persistency module)
- `/pullpiri/src/common/build.rs` (added persistency.proto)
- `/pullpiri/src/common/Cargo.toml` (removed etcd-client)
- `/pullpiri/src/server/Cargo.toml` (added persistency-service to workspace)