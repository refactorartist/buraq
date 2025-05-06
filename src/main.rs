use env_logger;
use anyhow;
use actix_web::web;

use buraq::config::AppConfig; 


#[actix_web::main]
async fn main() -> Result<(), anyhow::Error> {
    dotenvy::dotenv()?;
    env_logger::init();  

    let app_config = AppConfig::from_env().await?; 

    let _app_data = web::Data::new(
        app_config 
    );
    
    
    Ok(())
}