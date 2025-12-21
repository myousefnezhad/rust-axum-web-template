use app_config::AppConfig;
use dotenv::dotenv;

fn main() {
    dotenv().ok();
    let config = AppConfig::new();
    println!("{:#?}", &config);
}
