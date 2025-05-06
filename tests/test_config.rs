use buraq::config::AppConfig;

#[test]
fn test_app_config_from_env() {
    // Test successful config loading
    temp_env::with_vars(
        vec![
            ("BURAQ_DATABASE_URI", Some("mongodb://localhost:27017")),
            ("BURAQ_HOST", Some("127.0.0.1")),
            ("BURAQ_PORT", Some("8080")),
        ],
        || {
            let config = AppConfig::from_env(Some(false)).expect("Failed to load config");
            
            // Verify application config
            assert_eq!(config.application.host, "127.0.0.1");
            assert_eq!(config.application.port, 8080);
            
            // Verify database client is created
            assert_eq!(config.application.database_uri, "mongodb://localhost:27017");
            
        }
    );
}

#[test]
fn test_app_config_missing_database_uri() {
    temp_env::with_vars(
        vec![
            ("BURAQ_HOST", Some("127.0.0.1")),
            ("BURAQ_PORT", Some("8080")),
        ],
        || {
            let result = AppConfig::from_env(Some(false));
            assert!(result.is_err());
        }
    );
}

#[test]
fn test_app_config_missing_host() {
    temp_env::with_vars(
        vec![
            ("BURAQ_DATABASE_URI", Some("mongodb://localhost:27017")),
            ("BURAQ_PORT", Some("8080")),
        ],
        || {
            let result = AppConfig::from_env(Some(false));
            assert!(result.is_err());
        }
    );
}


#[test]
fn test_app_config_missing_port() {
    temp_env::with_vars(
        vec![
            ("BURAQ_DATABASE_URI", Some("mongodb://localhost:27017")),
            ("BURAQ_HOST", Some("127.0.0.1")),
        ],
        || {
            let result = AppConfig::from_env(Some(false));
            assert!(result.is_err());
        }
    );
}



