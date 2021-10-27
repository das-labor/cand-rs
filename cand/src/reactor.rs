use std::process::Output;
use futures::{Stream, Sink, SinkExt, StreamExt};
use futures::channel::mpsc;
use labctl::cand;
use slotmap::DenseSlotMap;
use tokio::task::{self, JoinHandle};
use crate::util;
use crate::util::KillJoinHandle;

slotmap::new_key_type! {
    struct Task;
}

enum ReactorMessage {
    RegisterClient {
        read: Box<dyn Stream<Item=cand::Message> + Send + Unpin>,
        write: mpsc::UnboundedSender<cand::Message>,
        task: JoinHandle<()>
    },
    RegisterUplinkTemp {
        read: Box<dyn Stream<Item=cand::Message> + Send + Unpin>,
        write: mpsc::UnboundedSender<cand::Message>,
        task: JoinHandle<()>
    },
    TaskDied {
        task: Task
    },
    Message {
        source: Task,
        payload: cand::Message
    }
}

#[derive(Clone)]
pub struct ReactorHandle {
    sender: mpsc::Sender<ReactorMessage>
}

impl ReactorHandle {
    pub async fn register_client<In: 'static>(&mut self, read: In, write: mpsc::UnboundedSender<cand::Message>, task: JoinHandle<()>)
    where
        In: Stream<Item=cand::Message> + Send + Unpin,
    {
        self.sender.send(ReactorMessage::RegisterClient {
            read: Box::new(read),
            write,
            task,
        }).await;
    }

    pub async fn register_uplink<In: 'static>(&mut self, read: In, write: mpsc::UnboundedSender<cand::Message>, task: JoinHandle<()>)
        where
            In: Stream<Item=cand::Message> + Send + Unpin,
    {
        self.sender.send(ReactorMessage::RegisterUplinkTemp {
            read: Box::new(read),
            write,
            task,
        }).await;
    }
}

struct TaskData {
    uplink: bool,
    /// This is set to None, when the client is to be dropped.
    sink: mpsc::UnboundedSender<cand::Message>,
    /// This is set to None, when the client is to be dropped.
    supervisor: Option<KillJoinHandle<()>>,
    reader: Option<KillJoinHandle<()>>
}

pub struct Reactor {
    tasks: DenseSlotMap<Task, TaskData>,
    sender: mpsc::Sender<ReactorMessage>,
    receive: mpsc::Receiver<ReactorMessage>
}

impl Reactor {
    pub fn new() -> (Reactor, ReactorHandle) {
        let (tx, rx) = mpsc::channel(16);

        let r = Reactor {
            tasks: DenseSlotMap::with_key(),
            receive: rx,
            sender: tx.clone()
        };
        let mut rh = ReactorHandle {
            sender: tx
        };
        (r, rh)
    }

    pub async fn run(&mut self) {
        while let Some(message) = self.receive.next().await {
            match message {
                ReactorMessage::RegisterClient { read, write, task } => {
                    let data = TaskData {
                        uplink: false,
                        sink: write,
                        supervisor: None,
                        reader: None
                    };

                    let key = self.tasks.insert(data);

                    let shandle = task::spawn(supervise(task, self.sender.clone(), key));
                    let rhandle = task::spawn(read_task(read, self.sender.clone(), key));

                    let data = self.tasks.get_mut(key).unwrap();
                    data.supervisor = Some(util::kill_task_on_drop(shandle));
                    data.reader = Some(util::kill_task_on_drop(rhandle));
                }
                ReactorMessage::TaskDied { task } => {
                    if let Some(data) = self.tasks.get(task) {
                        if data.uplink {
                            todo!("Respawn uplink")
                        }
                    }
                    // This should drop all task handles and therefore kill all tasks involved with this client
                    // which in turn should fully close the client connection, if it didn't happen already
                    self.tasks.remove(task);
                }
                ReactorMessage::Message { source, payload } => {
                    let from_uplink = self.tasks.get(source).map(|task| task.uplink);
                    if let Some(from_uplink) = from_uplink {
                        // The sending task actually exists
                        for (_, task) in &mut self.tasks {
                            if task.uplink != from_uplink {
                                task.sink.send(payload.clone()).await;
                            }
                        }
                    } // If the task does not exist (anymore), we just ignore the message
                }
                ReactorMessage::RegisterUplinkTemp { read, write, task } => {
                    let data = TaskData {
                        uplink: true,
                        sink: write,
                        supervisor: None,
                        reader: None
                    };

                    let key = self.tasks.insert(data);

                    let shandle = task::spawn(supervise(task, self.sender.clone(), key));
                    let rhandle = task::spawn(read_task(read, self.sender.clone(), key));

                    let data = self.tasks.get_mut(key).unwrap();
                    data.supervisor = Some(util::kill_task_on_drop(shandle));
                    data.reader = Some(util::kill_task_on_drop(rhandle));
                }
            }
        }
        log::warn!("Reactor exiting!")
    }
}

async fn read_task(mut read: Box<dyn Stream<Item=cand::Message> + Send + Unpin>, mut sender: mpsc::Sender<ReactorMessage>, key: Task) {
    while let Some(message) = read.next().await {
        sender.send(ReactorMessage::Message {
            source: key,
            payload: message
        }).await;
    }

}

async fn supervise(task: JoinHandle<()>, mut sender: mpsc::Sender<ReactorMessage>, key: Task) {
    let task = util::kill_task_on_drop(task);
    task.await;
    sender.send(ReactorMessage::TaskDied {
        task: key
    }).await;
}