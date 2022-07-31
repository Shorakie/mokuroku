use serenity::{
    async_trait,
    client::{Context, EventHandler},
    model::{event::ResumedEvent, gateway::Ready},
};
use tracing::{debug, info};

pub struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        info!("Connected as {}!", ready.user.name);
    }

    async fn resume(&self, _: Context, resume: ResumedEvent) {
        debug!("Resumed; trace: {:?}", resume.trace);
    }
}
