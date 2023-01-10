use crate::{database::ConcurrentNodeDatabase, util::execution::Runnable};
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use anyhow::Result;
use spec::{
    types::{Block, Transaction},
    Database as SpecDatabase,
};

pub struct Api {
    port: u16,
    database: ConcurrentNodeDatabase,
}

impl Runnable for Api {
    fn run(&self) -> Result<()> {
        start_server(self.port, &self.database)
    }
}

impl Api {
    pub fn new(port: u16, database: &ConcurrentNodeDatabase) -> Api {
        Api {
            port,
            database: database.clone(),
        }
    }
}

#[actix_web::main]
async fn start_server(port: u16, database: &ConcurrentNodeDatabase) -> Result<()> {
    let url = format!("localhost:{}", port);
    let state = web::Data::new(database.clone());

    HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .route("/block_template", web::get().to(get_block_template))
            .route("/blocks", web::get().to(get_blocks))
            .route("/blocks", web::post().to(add_block))
            .route("/transactions", web::get().to(get_transactions))
            .route("/transactions", web::post().to(add_transaction))
    })
    .bind(url)
    .unwrap()
    .run()
    .await?;

    Ok(())
}

async fn get_block_template(database: web::Data<ConcurrentNodeDatabase>) -> impl Responder {
    let template_block = Block::new_template(database.as_ref());

    HttpResponse::Ok().json(&template_block)
}

// Return list of all blocks in blockchain
async fn get_blocks(database: web::Data<ConcurrentNodeDatabase>) -> impl Responder {
    let blocks = database.get_all_blocks();

    HttpResponse::Ok().json(&blocks)
}

// Add new block to blockchain
async fn add_block(
    database: web::Data<ConcurrentNodeDatabase>,
    block_json: web::Json<Block>,
) -> HttpResponse {
    let block = block_json.into_inner();
    let result = database.append_block(&block);

    match result {
        Ok(_) => {
            info!("Received new block {}", block.index);
            HttpResponse::Ok().finish()
        }
        Err(error) => HttpResponse::BadRequest().body(error.to_string()),
    }
}

// Return list of all transactions not included into block
async fn get_transactions(database: web::Data<ConcurrentNodeDatabase>) -> impl Responder {
    let transactions = database.get_mempool_transactions();
    HttpResponse::Ok().json(&transactions)
}

// Add new transaction to pending pool
async fn add_transaction(
    database: web::Data<ConcurrentNodeDatabase>,
    json_transaction: web::Json<Transaction>,
) -> impl Responder {
    let transaction = json_transaction.into_inner();
    let result = database.add_transaction(transaction);
    match result {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(error) => HttpResponse::BadRequest().body(error.to_string()),
    }
}
