use poem::{get, handler, EndpointExt, Route};
use poem_ratelimit::{RateLimiter, RateLimiterImpl};
use redis::aio::ConnectionManager;

#[handler]
fn hello() -> String {
    "Hello".to_string()
}

#[allow(dead_code)]
async fn config(
    redis_addr: &str,
) -> Result<RateLimiterImpl<ConnectionManager, Route>, anyhow::Error> {
    let client = redis::Client::open(redis_addr)?;
    let config = {
        let data = std::fs::read_to_string("./rate_limit.yaml")?;
        serde_yaml::from_str(&data)?
    };
    let rate_limiter = RateLimiter::new(ConnectionManager::new(client).await?, config);

    let app = Route::new()
        .at("/", get(hello))
        .at("/hello", get(hello))
        .with(rate_limiter);

    Ok(app)
}

#[cfg(test)]
mod tests {
    use poem::{listener::TcpListener, Server};
    use testcontainers::runners::AsyncRunner;
    use testcontainers_modules::redis::Redis;
    use tokio::time::{sleep, Duration};

    use crate::config;

    #[tokio::test]
    async fn test_ratelimiting() {
        let _ = pretty_env_logger::try_init();

        // Start a Redis container
        let node = Redis.start().await.unwrap();
        let host_ip = node.get_host().await.unwrap();
        let host_port = node.get_host_port_ipv4(6379).await.unwrap();
        let url = format!("redis://{host_ip}:{host_port}");

        // Start the server
        let app = config(&url).await.unwrap();
        tokio::task::spawn(async move {
            if let Err(e) = Server::new(TcpListener::bind("127.0.0.1:3000"))
                .run(app)
                .await
            {
                println!("Server error: {e}");
            }
        });

        // Give the server a second to start
        sleep(Duration::from_secs(1)).await;

        let client = reqwest::Client::new();
        let url = "http://127.0.0.1:3000";

        // Send requests to test route-level rate limiting
        for i in 1..7 {
            let res = client.get(url).send().await.unwrap();
            println!("Request {}: Status: {}", i, res.status());
        }

        // Send requests to test IP-level rate limiting
        for i in 1..12 {
            let res = client.get(&format!("{}/hello", url)).send().await.unwrap();
            println!("Request {}: Status: {}", i, res.status());
        }

        // Additional request to check if rate limiting is enforced
        let res = client.get(url).send().await.unwrap();
        assert_eq!(res.status(), 429);
    }
}
