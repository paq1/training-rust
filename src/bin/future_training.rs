use async_trait::async_trait;
use futures::lock::Mutex;
use std::collections::HashMap;
use std::sync::mpsc::{channel, Sender};
use std::sync::Arc;
use std::thread::sleep;
use std::time::Duration;

use training_rust::shared::ResultErr;
use training_rust::shared::TrainingError::Simple;

type CorrelationID = String;

pub struct Subscriber<RESPONSE>
where
    RESPONSE: Send,
{
    pub data: Arc<Mutex<HashMap<CorrelationID, Sender<RESPONSE>>>>,
}

#[async_trait]
pub trait CanSubscribe<RESPONSE>
where
    RESPONSE: Send,
{
    async fn subscribe(&self, correlation_id: &str, sender: Sender<RESPONSE>) -> ResultErr<()>;
    async fn send(&self, correlation_id: &str, message: RESPONSE) -> ResultErr<()>;
}

#[async_trait]
impl<RESPONSE> CanSubscribe<RESPONSE> for Subscriber<RESPONSE>
where
    RESPONSE: Send,
{
    async fn subscribe(&self, correlation_id: &str, sender: Sender<RESPONSE>) -> ResultErr<()> {
        let mut lock = self.data.lock().await;
        // Stocke un Arc<Sender> directement
        lock.insert(correlation_id.to_string(), sender);
        Ok(())
    }

    async fn send(&self, correlation_id: &str, message: RESPONSE) -> ResultErr<()> {
        let lock = self.data.lock().await;

        if let Some(sender_arc) = lock.get(correlation_id) {
            // Cloner l'Arc<Sender> pour envoyer le message
            sender_arc
                .send(message)
                .map_err(|e| Simple("erreur lors de l'envoi".to_string()))?;
        } else {
            return Err(Simple(
                format!("Aucun Sender trouvé pour l'ID '{}'", correlation_id).into(),
            ));
        }

        Ok(())
    }
}

#[tokio::main]
async fn main() -> ResultErr<()> {
    let subscriber: Arc<Subscriber<String>> = Arc::new(Subscriber {
        data: Arc::new(Mutex::new(HashMap::new())),
    });

    let (tx, mut rx) = channel();
    subscriber.subscribe("123", tx).await?;

    // simule te traitement async du lister qui fera la taf !
    let cloned_subscriber = subscriber.clone();
    tokio::spawn(async move {
        sleep(Duration::from_secs(5));
        cloned_subscriber
            .send("123", "pouet pouet".to_string())
            .await
            .expect("Erreur lors de l'envoi");
    });

    rx.recv_timeout(Duration::from_secs(30))
        .map(|msg| println!("Reçu : {}", msg))
        .map_err(|e| Simple(format!("Erreur lors de l'envoi: {}", e)))
}
