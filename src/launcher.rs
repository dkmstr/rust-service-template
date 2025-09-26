use std::sync::Arc;
use tokio::sync::Notify;

pub fn run(stop: Arc<Notify>) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async move {
        tokio::select! {
            _ = async_main() => {},
            _ = stop.notified() => {
                // lógica de shutdown
                println!("Recibido STOP, cerrando...");
            }
        }
    });
}

async fn async_main() {
    // tu lógica principal
    loop {
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        println!("trabajando...");
    }
}
