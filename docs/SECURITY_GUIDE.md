# Security Guide: Handling API Credentials

This guide covers best practices for securely handling API keys and secrets when using the LotuSX trading library.

## 🔒 Security Features

### Built-in Protection
- **Memory Protection**: Credentials are stored using the `secrecy` crate, which provides memory protection
- **No Accidental Logging**: Credentials are automatically redacted in debug output and serialization
- **Multiple Configuration Methods**: Support for environment variables, direct instantiation, and read-only modes

## 📝 Configuration Methods

### 1. Environment Variables (Recommended)

The safest way to handle credentials is through environment variables:

```bash
# Set your credentials
export BINANCE_API_KEY="your_api_key_here"
export BINANCE_SECRET_KEY="your_secret_key_here"
export BINANCE_TESTNET="true"  # Optional, defaults to false
export BINANCE_BASE_URL="https://custom.api.url"  # Optional
```

```rust
use lotusx::core::config::ExchangeConfig;
use lotusx::exchanges::binance::BinanceConnector;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load configuration from environment variables
    let config = ExchangeConfig::from_env("BINANCE")?;
    let connector = BinanceConnector::new(config);
    
    // Your trading logic here...
    Ok(())
}
```

### 2. Direct Configuration

For development or when environment variables aren't suitable:

```rust
use lotusx::core::config::ExchangeConfig;

let config = ExchangeConfig::new(
    "your_api_key".to_string(),
    "your_secret_key".to_string(),
)
.testnet(true);  // Always use testnet for development
```

### 3. Read-Only Mode

For market data only (no trading operations):

```rust
use lotusx::core::config::ExchangeConfig;

let config = ExchangeConfig::read_only()
    .testnet(true);

// This config can only be used for public endpoints
// like getting markets, market data streaming, etc.
```

## 🛡️ Security Best Practices

### 1. Environment Variables
- ✅ **DO**: Use environment variables for production
- ✅ **DO**: Use different credentials for testnet and mainnet
- ✅ **DO**: Rotate your API keys regularly
- ❌ **DON'T**: Hardcode credentials in source code
- ❌ **DON'T**: Commit `.env` files to version control

### 2. API Key Permissions
Configure your exchange API keys with minimal required permissions:

- **For Trading Bots**: Enable only "Trade" permissions
- **For Market Data**: Use read-only keys when possible
- **For Monitoring**: Enable only "Read" permissions
- **Never Enable**: Withdrawal permissions unless absolutely necessary

### 3. Network Security
```rust
// Always use testnet during development
let config = ExchangeConfig::from_env("BINANCE")?
    .testnet(true);

// For production, ensure you're using HTTPS endpoints
let config = ExchangeConfig::from_env("BINANCE")?
    .base_url("https://api.binance.com".to_string());
```

### 4. Error Handling
```rust
use lotusx::core::config::{ExchangeConfig, ConfigError};

match ExchangeConfig::from_env("BINANCE") {
    Ok(config) => {
        if config.has_credentials() {
            // Safe to use for authenticated operations
            println!("Credentials loaded successfully");
        } else {
            // Only public endpoints available
            println!("Running in read-only mode");
        }
    }
    Err(ConfigError::MissingEnvironmentVariable(var)) => {
        eprintln!("Missing required environment variable: {}", var);
        eprintln!("Please set {} before running", var);
    }
    Err(e) => {
        eprintln!("Configuration error: {}", e);
    }
}
```

## 📁 File-based Configuration

### Using .env Files

Create a `.env` file (but **never commit it**):

```bash
# .env file (ADD TO .gitignore!)
BINANCE_API_KEY=your_api_key_here
BINANCE_SECRET_KEY=your_secret_key_here
BINANCE_TESTNET=true
```

#### **Method 1: Using the env-file Feature (Recommended)**

Add the env-file feature to your `Cargo.toml`:

```toml
[dependencies]
lotusx = { version = "0.1", features = ["env-file"] }
```

Then in your code:

```rust
use lotusx::core::config::ExchangeConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Automatically loads from .env file then environment variables
    let config = ExchangeConfig::from_env_file("BINANCE")?;
    
    // Or with custom path
    let config = ExchangeConfig::from_env_file_with_path("BINANCE", ".env.development")?;
    
    // Or with automatic detection (.env.local, .env.development, .env)
    let config = ExchangeConfig::from_env_auto("BINANCE")?;
    
    // Your trading logic here...
    Ok(())
}
```

#### **Method 2: Using External dotenv Crate**

Load it manually using the `dotenv` crate:

```rust
// Add to Cargo.toml: dotenv = "0.15"
use dotenv::dotenv;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok(); // Load .env file
    
    let config = ExchangeConfig::from_env("BINANCE")?;
    // ... rest of your code
}
```

#### **Complete .env File Example**

Create this as `.env.example` (safe to commit):

```bash
# LotuSX Configuration Example
# Copy this file to .env and fill in your actual API credentials
# NEVER commit the actual .env file to version control!

# Binance Configuration
BINANCE_API_KEY=your_binance_api_key_here
BINANCE_SECRET_KEY=your_binance_secret_key_here
BINANCE_TESTNET=true
BINANCE_BASE_URL=https://testnet.binance.vision

# Other Exchange Examples (uncomment and configure as needed)

# Binance US
# BINANCE_US_API_KEY=your_binance_us_api_key
# BINANCE_US_SECRET_KEY=your_binance_us_secret_key
# BINANCE_US_TESTNET=false
# BINANCE_US_BASE_URL=https://api.binance.us

# Environment Settings
ENVIRONMENT=development
LOG_LEVEL=info

# Security Notes:
# 1. Add .env to your .gitignore file
# 2. Use different credentials for development/production
# 3. Never share .env files via email or chat
# 4. Set file permissions: chmod 600 .env
# 5. Rotate your API keys regularly
```

#### **Multiple Environment Support**

You can use different .env files for different environments:

```bash
# Project structure
.env.example          # Template (safe to commit)
.env                  # Default environment (DO NOT COMMIT)
.env.local            # Local overrides (DO NOT COMMIT)
.env.development      # Development settings (DO NOT COMMIT)  
.env.production       # Production settings (DO NOT COMMIT)
```

Usage:

```rust
#[cfg(feature = "env-file")]
{
    // Load with priority: .env.local > .env.development > .env
    let config = ExchangeConfig::from_env_auto("BINANCE")?;
    
    // Or load specific environment
    let config = ExchangeConfig::from_env_file_with_path("BINANCE", ".env.production")?;
}
```

#### **Setup Instructions for Users**

1. **Copy the example file:**
   ```bash
   cp .env.example .env
   ```

2. **Edit with your credentials:**
   ```bash
   # Edit .env file with your actual API keys
   nano .env  # or vim .env, or your preferred editor
   ```

3. **Set secure permissions:**
   ```bash
   chmod 600 .env  # Read/write for owner only
   ```

4. **Add to .gitignore:**
   ```bash
   echo ".env" >> .gitignore
   echo ".env.*" >> .gitignore
   echo "!.env.example" >> .gitignore
   ```

5. **Run your application:**
   ```bash
   # With env-file feature
   cargo run --features env-file
   
   # Or if you added it to default features
   cargo run
   ```

## 🚨 Common Security Mistakes

### ❌ DON'T DO THESE:

```rust
// NEVER hardcode credentials
let config = ExchangeConfig::new(
    "pk_12345abcdef".to_string(),  // ❌ DON'T
    "sk_98765fedcba".to_string(),  // ❌ DON'T
);

// NEVER log credentials
println!("API Key: {}", config.api_key()); // ❌ DON'T

// NEVER use production keys for testing
let config = ExchangeConfig::new(api_key, secret_key)
    .testnet(false); // ❌ DON'T in development

// NEVER commit secrets to git
git add .env  // ❌ DON'T
```

### ✅ DO THESE INSTEAD:

```rust
// ✅ Use environment variables
let config = ExchangeConfig::from_env("BINANCE")?;

// ✅ Check credentials are loaded
if !config.has_credentials() {
    return Err("No credentials available".into());
}

// ✅ Use testnet for development
let config = ExchangeConfig::from_env("BINANCE")?
    .testnet(true);

// ✅ Add .env to .gitignore
echo ".env" >> .gitignore
```

## 🧪 Testing Safely

### Test Configuration
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_only_config() {
        let config = ExchangeConfig::read_only();
        assert!(!config.has_credentials());
        assert_eq!(config.testnet, false);
    }

    #[tokio::test]
    async fn test_with_mock_credentials() {
        // Use fake credentials for testing
        let config = ExchangeConfig::new(
            "test_api_key".to_string(),
            "test_secret_key".to_string(),
        ).testnet(true);
        
        // Test your logic with testnet
        let connector = BinanceConnector::new(config);
        // ... your tests
    }
}
```

## 📋 Deployment Checklist

Before deploying to production:

- [ ] API keys are stored in environment variables
- [ ] `.env` files are in `.gitignore`
- [ ] Using separate testnet/mainnet credentials
- [ ] API keys have minimal required permissions
- [ ] Credentials are rotated regularly
- [ ] Error handling for missing credentials
- [ ] Logging doesn't expose sensitive data
- [ ] Network connections use HTTPS
- [ ] Rate limiting is implemented
- [ ] Monitoring and alerting in place

## 🔄 Credential Rotation

Implement regular credential rotation:

```rust
use std::time::{SystemTime, UNIX_EPOCH, Duration};

const KEY_ROTATION_DAYS: u64 = 30;

fn should_rotate_key(created_timestamp: u64) -> bool {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    let key_age = now - created_timestamp;
    key_age > KEY_ROTATION_DAYS * 24 * 3600
}

// Example usage in your monitoring system
if should_rotate_key(key_created_at) {
    eprintln!("WARNING: API key is {} days old and should be rotated", 
              (key_age / (24 * 3600)));
}
```

## 📞 Need Help?

If you discover a security vulnerability, please:

1. **DO NOT** create a public GitHub issue
2. Email the maintainers directly
3. Include details about the vulnerability
4. Allow time for the fix before public disclosure

Remember: Security is a shared responsibility. Always follow the principle of least privilege and defense in depth. 