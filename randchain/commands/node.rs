use super::super::utils;
use clap::ArgMatches;
use std::net::SocketAddr;
use sync::{create_local_sync_node, create_sync_connection_factory, create_sync_peers};
use {p2p, LOG_INFO, PROTOCOL_MINIMUM, PROTOCOL_VERSION};

pub fn start(matches: &ArgMatches) -> Result<(), String> {
    // parse matches into Config
    let cfg = utils::config::parse(matches)?;

    // init logs
    if !cfg.quiet {
        if cfg!(windows) {
            logs::init(LOG_INFO, logs::DateLogFormatter);
        } else {
            logs::init(LOG_INFO, logs::DateAndColorLogFormatter);
        }
    } else {
        env_logger::init();
    }
    // init event loop
    let mut el = p2p::event_loop();
    // init database
    utils::init_db(&cfg)?;
    // init account path
    let account_dir = utils::create_account_dir(cfg.data_dir.clone());
    // TODO: init account
    // init node table path
    let nodes_path = utils::create_node_table(cfg.data_dir.clone());
    // init p2p config
    let p2p_cfg = p2p::Config {
        threads: cfg.p2p_threads,
        inbound_connections: cfg.inbound_connections,
        outbound_connections: cfg.outbound_connections,
        connection: p2p::NetConfig {
            protocol_version: PROTOCOL_VERSION,
            protocol_minimum: PROTOCOL_MINIMUM,
            magic: cfg.network.magic(),
            local_address: SocketAddr::new(cfg.host, cfg.port),
            services: cfg.services,
            user_agent: cfg.user_agent,
            start_height: 0,
            relay: true,
            network: cfg.network,
        },
        peers: cfg.peers,
        seeds: cfg.seednodes,
        node_table_path: nodes_path,
        preferable_services: cfg.services,
        internet_protocol: cfg.internet_protocol,
    };

    let sync_peers = create_sync_peers();
    let local_sync_node = create_local_sync_node(
        cfg.network,
        cfg.db.clone(),
        sync_peers.clone(),
        cfg.verification_params,
    );
    let sync_connection_factory =
        create_sync_connection_factory(sync_peers.clone(), local_sync_node.clone());

    if let Some(block_notify_command) = cfg.block_notify_command {
        local_sync_node
            .install_sync_listener(Box::new(utils::BlockNotifier::new(block_notify_command)));
    }

    let p2p =
        p2p::P2P::new(p2p_cfg, sync_connection_factory, el.handle()).map_err(|x| x.to_string())?;
    let rpc_deps = utils::rpc::Dependencies {
        network: cfg.network,
        storage: cfg.db,
        local_sync_node: local_sync_node.clone(),
        p2p_context: p2p.context().clone(),
        remote: el.remote(),
    };
    let _rpc_server = utils::rpc::new_http_rpc_server(cfg.rpc_config, rpc_deps)?;

    p2p.run().map_err(|_| "Failed to start p2p module")?;
    el.run(p2p::forever()).unwrap();
    Ok(())
}
