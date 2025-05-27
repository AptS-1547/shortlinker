use actix_web::{get, App, HttpResponse, HttpServer, Responder, web};
use dotenv::dotenv;
use std::env;
use log::info;

// 配置结构体
#[derive(Clone, Debug)]
struct Config {
    server_host: String,
    server_port: u16,
}

#[get("/{path}*")]
async fn shortlinker(tail: web::Path<String>) -> impl Responder {
    let captured_path = tail.to_string();

    if captured_path == "" {
        info!("跳转到主页");
        return HttpResponse::TemporaryRedirect()
            .append_header(("Location", "https://www.esaps.net/"))
            .finish();
    } else {
        return HttpResponse::Ok()
            .content_type("text/plain")
            .body(format!("你访问的路径是: {}", captured_path));
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // 加载环境变量
    dotenv().ok();

    //设置 logger
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));
    
    // 从环境变量读取配置
    let config = Config {
        server_host: env::var("SERVER_HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
        server_port: env::var("SERVER_PORT").unwrap_or_else(|_| "8080".to_string()).parse().unwrap(),
    };
    
    let bind_address = format!("{}:{}", config.server_host, config.server_port);
    info!("启动服务器，监听 {}", bind_address);
    
    // 启动服务器
    HttpServer::new(move || {
        App::new()
            .service(shortlinker)
    })
    .bind(bind_address)?
    .run()
    .await
}