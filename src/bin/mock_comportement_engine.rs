use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use async_trait::async_trait;
use futures::channel::oneshot::Sender;
use futures::lock::Mutex;
use futures::SinkExt;
use training_rust::shared::{ResultErr, TrainingError};

type CorrelationID = String;

#[async_trait]
trait CanSubscribe<RESPONSE: Send>: Send + Sync {
    async fn subscribe(&mut self, correlation_id: &str, sender: Sender<RESPONSE>) -> ResultErr<()>;
    async fn send(&self, correlation_id: &str, message: RESPONSE) -> ResultErr<()>;
}

struct Subscriber<RESPONSE> {
    pub data: Arc<Mutex<HashMap<CorrelationID, Arc<Mutex<Sender<RESPONSE>>>>>>,
}

#[async_trait]
impl<RESPONSE: Send> CanSubscribe<RESPONSE> for Subscriber<RESPONSE> {
    async fn subscribe(&mut self, correlation_id: &str, sender: Arc<Mutex<Sender<RESPONSE>>>) -> ResultErr<()> {
        self.data.lock().await.insert(correlation_id.to_string(), sender);
        Ok(())
    }

    async fn send(&self, correlation_id: &str, message: RESPONSE) -> ResultErr<()> {
        let lock = self.data.lock().await;

        if let Some(sender_arc) = lock.get(correlation_id) { // Utiliser `get` ici
            let mut sender = sender_arc.lock().await; // Verrouiller le Sender via Arc<Mutex>
            sender
                .send(message)
                .await
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;
        } else {
            return Err(format!("Aucun Sender trouvé pour l'ID '{}'", correlation_id).into());
        }
        Ok(())
    }

}

struct Engine {

}

impl Engine {
    pub async fn compute(&self) -> ResultErr<String> {
        // TODO : on cree un subscribe

        Err(TrainingError::Simple("Not implemented".to_string()))
    }
}

#[tokio::main]
async fn main() -> ResultErr<()> {
    println!("training future rust :)");

    Ok(())
    // let engine: Engine = Engine {};
    //
    // engine
    //     .compute().await
    //     .map(|_| ())
}
