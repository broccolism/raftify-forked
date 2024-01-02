#[macro_use]
extern crate slog;
extern crate slog_async;
extern crate slog_scope;
extern crate slog_term;

use memstore::{
    config::build_config,
    state_machine::{HashStore, LogEntry},
    utils::load_peers,
};
use raftify::{
    raft::derializer::set_custom_deserializer, AbstractLogEntry, ClusterJoinTicket, MyDeserializer,
    Raft,
};
use slog::Drain;

use actix_web::{get, web, App, HttpServer, Responder};
use slog_envlogger::LogBuilder;
use structopt::StructOpt;
// use slog_envlogger::LogBuilder;

#[derive(Debug, StructOpt)]
struct Options {
    #[structopt(long)]
    raft_addr: String,
    #[structopt(long)]
    peer_addr: Option<String>,
    #[structopt(long)]
    web_server: Option<String>,
    #[structopt(
        long,
        help = "Ignore cluster_config.toml's peers and bootstrap a new cluster"
    )]
    ignore_static_bootstrap: bool,
}

#[get("/put/{id}/{name}")]
async fn put(
    data: web::Data<(HashStore, Raft<LogEntry, HashStore>)>,
    path: web::Path<(u64, String)>,
) -> impl Responder {
    let log_entry = LogEntry::Insert {
        key: path.0,
        value: path.1.clone(),
    };
    data.1.raft_node.propose(log_entry.encode().unwrap()).await;

    "OK".to_string()
}

#[get("/get/{id}")]
async fn get(
    data: web::Data<(HashStore, Raft<LogEntry, HashStore>)>,
    path: web::Path<u64>,
) -> impl Responder {
    let id = path.into_inner();

    let response = data.0.get(id);
    format!("{:?}", response)
}

#[get("/leader")]
async fn leader_id(data: web::Data<(HashStore, Raft<LogEntry, HashStore>)>) -> impl Responder {
    let raft = data.clone();
    let leader_id = raft.1.raft_node.get_leader_id().await.to_string();
    format!("{:?}", leader_id)
}

#[get("/leave")]
async fn leave(data: web::Data<(HashStore, Raft<LogEntry, HashStore>)>) -> impl Responder {
    let raft = data.clone();
    raft.1.raft_node.clone().leave().await;
    "OK".to_string()
}

#[actix_rt::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let decorator = slog_term::TermDecorator::new().build();
    let drain = slog_term::FullFormat::new(decorator).build().fuse();
    let drain = slog_async::Async::new(drain).build().fuse();

    let mut builder = LogBuilder::new(drain);
    builder = builder.filter(None, slog::FilterLevel::Debug);

    if let Ok(s) = std::env::var("RUST_LOG") {
        builder = builder.parse(&s);
    }
    let drain = builder.build();

    let logger = slog::Logger::root(drain, o!());

    set_custom_deserializer(MyDeserializer::<LogEntry, HashStore>::new());

    // converts log to slog
    // let _scope_guard = slog_scope::set_global_logger(logger.clone());
    let _log_guard = slog_stdlog::init_with_level(log::Level::Debug).unwrap();

    let options = Options::from_args();
    let store = HashStore::new();

    let peers = match options.ignore_static_bootstrap {
        true => None,
        false => {
            let peers = load_peers().await?;
            Some(peers)
        }
    };

    let cfg = build_config();

    let (raft, raft_handle) = match options.peer_addr {
        Some(peer_addr) => {
            log::info!("Running in Follower mode");

            let mut request_id_resp: Option<ClusterJoinTicket> = None;

            let node_id = match peers {
                Some(ref peers) => peers
                    .get_node_id_by_addr(options.raft_addr.clone())
                    .unwrap(),
                None => {
                    request_id_resp = Raft::<LogEntry, HashStore>::request_id(peer_addr.clone())
                        .await
                        .ok();
                    request_id_resp.to_owned().unwrap().reserved_id
                }
            };

            let mut raft = Raft::<LogEntry, HashStore>::build(
                node_id,
                options.raft_addr,
                store.clone(),
                cfg,
                logger.clone(),
                peers.clone(),
            )?;

            let handle = tokio::spawn(raft.clone().run());

            if let Some(request_id_resp) = request_id_resp {
                raft.join(request_id_resp).await;
            } else if let Some(peers) = peers {
                let leader_addr = peers.get(&1).unwrap().addr;
                raft.member_bootstrap_ready(leader_addr, node_id).await?;
            } else {
                unreachable!()
            }

            (raft, handle)
        }
        None => {
            log::info!("Bootstrap a Raft Cluster");
            let node_id = 1;
            let raft = Raft::build(
                node_id,
                options.raft_addr,
                store.clone(),
                cfg,
                logger.clone(),
                peers,
            )?;
            let handle = tokio::spawn(raft.clone().run());
            (raft, handle)
        }
    };

    if let Some(addr) = options.web_server {
        let _web_server = tokio::spawn(
            HttpServer::new(move || {
                App::new()
                    .app_data(web::Data::new((store.clone(), raft.clone())))
                    .service(put)
                    .service(get)
                    // .service(leave)
                    .service(leader_id)
            })
            .bind(addr)
            .unwrap()
            .run(),
        );
    }

    let result = tokio::try_join!(raft_handle)?;
    result.0?;
    Ok(())
}
