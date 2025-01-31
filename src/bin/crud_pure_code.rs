use async_trait::async_trait;
use futures::lock::{Mutex, MutexGuard};
use std::collections::HashMap;
use std::sync::Arc;

pub type ResultErr<DATA> = Result<DATA, TrainingError>;

#[derive(Debug)]
pub enum TrainingError {
    Simple(String),
}


#[derive(Debug, Clone)]
pub struct Entity<T> {
    data: T,
    identifiant: String,
    version: u32,
}


#[async_trait]
pub trait Dao<T> {
    async fn get_by_id(&self, id: &str) -> ResultErr<Entity<T>>;

    async fn save(&self, item: &T) -> ResultErr<String>;
}

struct DaoHashMap<T> {
    datas: Arc<Mutex<HashMap<String, Entity<T>>>>,
}

impl<T> DaoHashMap<T> {
    fn new() -> Self {
        Self {
            datas: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl<T> Dao<T> for DaoHashMap<T>
where
    T: Send + Sync + Clone,
{
    async fn get_by_id(&self, id: &str) -> ResultErr<Entity<T>> {
        let mut guard = self.datas.lock().await;
        let c = guard.get(id);
        c.map(|e| Ok(e.clone())).unwrap_or(Err(TrainingError::Simple("missing".to_string())))
    }

    async fn save(&self, item: &T) -> ResultErr<String> {

        let id = uuid::Uuid::new_v4().to_string();

        let entity = Entity {
            data: item.clone(),
            identifiant: id.clone(),
            version: 1
        };

        let mut guard: MutexGuard<HashMap<String, Entity<T>>> = self.datas.lock().await;
        guard.insert(id.clone(), entity);

        Ok(id)
    }
}


async fn change_something(dao: Arc<dyn Dao<String>>) {

}


#[tokio::main]
async fn main() -> ResultErr<()> {
    println!("Hello, world!");
    let dao: Arc<dyn Dao<String>> = Arc::new(DaoHashMap::new());
    let id = dao.save(&"mon item".to_string()).await?;
    println!("id: {id}");
    let stored_entity = dao.get_by_id(id.as_str()).await?;
    println!("entity : {stored_entity:?}");
    Ok(())
}