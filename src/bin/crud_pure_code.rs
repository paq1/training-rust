use async_trait::async_trait;
use futures::lock::Mutex;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

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
    async fn get_by_id(&self, id: &str) -> ResultErr<Option<Entity<T>>>;

    async fn save(&self, item: &T) -> ResultErr<String>;

    async fn upsert(&self, item: &Entity<T>) -> ResultErr<String>;
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
    async fn get_by_id(&self, id: &str) -> ResultErr<Option<Entity<T>>> {
        let mut guard = self.datas.lock().await;
        let c = guard.get(id);
        c.map(|e| Ok(Some(e.clone()))).unwrap_or(Ok(None))
    }

    async fn save(&self, item: &T) -> ResultErr<String> {

        let id: String = Uuid::new_v4().to_string();

        let entity = Entity {
            data: item.clone(),
            identifiant: id.clone(),
            version: 1
        };

        self.upsert(&entity).await
    }

    async fn upsert(&self, item: &Entity<T>) -> ResultErr<String> {
        let identifiant = item.identifiant.clone();
        let mut guard = self.datas.lock().await;
        guard.insert(identifiant.clone(), item.clone());
        Ok(identifiant)
    }
}


async fn change_something(dao: Arc<dyn Dao<String>>) -> ResultErr<()> {
    let inserted = dao.save(&"hello world".to_string()).await?;
    println!("{:?}", inserted.as_str());
    let data = dao.get_by_id(inserted.as_str()).await?;
    if data.is_none() {
        println!("data is None");
    } else {
        println!("data is Some");
    }

    Ok(())
}


#[tokio::main]
async fn main() -> ResultErr<()> {
    println!("Hello, world!");
    let dao: Arc<dyn Dao<String>> = Arc::new(DaoHashMap::new());
    let id = dao.save(&"mon item".to_string()).await?;
    println!("id: {id}");
    let stored_entity = dao.get_by_id(id.as_str()).await?;
    println!("entity : {stored_entity:?}");

    change_something(dao.clone()).await?;
    change_something(dao).await?;

    Ok(())
}