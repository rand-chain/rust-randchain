use primitives::hash::H256;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;
use std::thread;
use sync::SyncListener;

enum BlockNotifierTask {
    NewBlock(H256),
    Stop,
}

pub struct BlockNotifier {
    tx: Sender<BlockNotifierTask>,
    is_synchronizing: Arc<AtomicBool>,
    worker_thread: Option<thread::JoinHandle<()>>,
}

impl BlockNotifier {
    pub fn new(block_notify_command: String) -> Self {
        let (tx, rx) = channel();
        let is_synchronizing = Arc::new(AtomicBool::default());
        BlockNotifier {
            tx: tx,
            is_synchronizing: is_synchronizing.clone(),
            worker_thread: Some(
                thread::Builder::new()
                    .name("Block notification thread".to_owned())
                    .spawn(move || BlockNotifier::worker(rx, block_notify_command))
                    .expect("Error creating block notification thread"),
            ),
        }
    }

    fn worker(rx: Receiver<BlockNotifierTask>, block_notify_command: String) {
        for cmd in rx {
            match cmd {
                BlockNotifierTask::NewBlock(new_block_hash) => {
                    let new_block_hash = new_block_hash.to_reversed_str();
                    let command = block_notify_command.replace("%s", &new_block_hash);
                    let c_command = ::std::ffi::CString::new(command.clone()).unwrap();
                    unsafe {
                        use libc::system;

                        let err = system(c_command.as_ptr());
                        if err != 0 {
                            error!(target: "randchain", "Block notification command {} exited with error code {}", command, err);
                        }
                    }
                }
                BlockNotifierTask::Stop => break,
            }
        }
        trace!(target: "randchain", "Block notification thread stopped");
    }
}

impl SyncListener for BlockNotifier {
    fn synchronization_state_switched(&self, is_synchronizing: bool) {
        self.is_synchronizing
            .store(is_synchronizing, Ordering::SeqCst);
    }

    fn best_storage_block_inserted(&self, block_hash: &H256) {
        if !self.is_synchronizing.load(Ordering::SeqCst) {
            self.tx
                .send(BlockNotifierTask::NewBlock(block_hash.clone()))
                .expect("Block notification thread have the same lifetime as `BlockNotifier`")
        }
    }
}

impl Drop for BlockNotifier {
    fn drop(&mut self) {
        if let Some(join_handle) = self.worker_thread.take() {
            let _ = self.tx.send(BlockNotifierTask::Stop);
            join_handle.join().expect("Clean shutdown.");
        }
    }
}
