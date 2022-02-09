
use crate::tuner_types::{Match,MatchResult,TimeControl,InputParser};
use crate::json_config::Engine;

use once_cell::sync::Lazy;
use rchess_engine_lib::explore::AtomicBool;

use std::str::FromStr;
use std::io::BufReader;
use std::io::{self,Write,BufRead,Stdout,Stdin};
use std::process::{Command,Stdio, Child};
use std::sync::Arc;

use crossbeam::channel::{Sender,Receiver};

// static PANIC_INIT: Lazy<()> = Lazy::new(|| {
//     let hook = std::panic::take_hook();
//     std::panic::set_hook(Box::new(move |panicinfo| {
//         let loc = panicinfo.location();
//         let mut file = std::fs::File::create("/home/me/code/rust/rchess/panic.log").unwrap();
//         let s = format!("Panicking, Location: {:?}", loc);
//         file.write(s.as_bytes()).unwrap();
//         // cutechess doesn't kill child processes when parent panics
//         let child = Command::new("pkill")
//             .args(["rchess_uci"])
//             .spawn()
//             .unwrap();
//         hook(panicinfo)
//     }));
// });

#[derive(Debug,Clone)]
pub struct Supervisor {
    engine:   Engine,

    tunable:  Tunable,

}

#[derive(Debug,PartialEq,Eq,PartialOrd,Ord,Clone)]
pub struct Tunable {
    name:    String,
    min:     u64,
    max:     u64,
    start:   u64,
    step:    u64,
}

#[derive(Debug,Clone)]
pub struct CuteChess {
    // pub child:       Arc<Option<Child>>,
    pub pid:         u32,
    pub children:    Vec<u32>,
    pub stop:        Arc<AtomicBool>,
    pub rx:          Arc<Receiver<Match>>,
}

impl Drop for CuteChess {
    fn drop(&mut self) {
        for child in self.children.iter() {
            println!("killing child pid {}", child);
            // std::process::Command::new("kill").arg(&format!("{}", child));
            nix::sys::signal::kill(
                nix::unistd::Pid::from_raw(*child as i32), nix::sys::signal::SIGKILL).unwrap();
        }
        println!("killing pid {}", self.pid);
        // std::process::Command::new("kill").arg(&format!("{}", self.pid));
        nix::sys::signal::kill(
            nix::unistd::Pid::from_raw(self.pid as i32), nix::sys::signal::SIGKILL).unwrap();
    }
}

impl CuteChess {

    pub fn run_cutechess(
        // engine1:          Engine,
        // engine2:          Engine,
        engine1:          &str,
        engine2:          &str,
        timecontrol:      TimeControl,
        output_label:     &str,
        num_games:        u64,
        (elo0,elo1):      (u64,u64),
        confidence:       f64,
    ) -> CuteChess {

        // let _ = PANIC_INIT

        let timestamp = get_timestamp();

        let output_file = &format!("tuning_logs/out_{}_{}.pgn", output_label, timestamp);
        let log_file    = &format!("tuning_logs/log_{}_{}.pgn", output_label, timestamp);

        let time = timecontrol.print();

        let args = [
            "-tournament gauntlet",
            "-concurrency 1",
            &format!("-pgnout {}", output_file),
            &format!("-engine conf={} {} timemargin=50 restart=off", engine1, time),
            &format!("-engine conf={} {} timemargin=50 restart=off", engine2, time),
            "-each proto=uci",
            "-openings file=tables/openings-10ply-100k.pgn policy=round",
            "-tb tables/syzygy/",
            "-tbpieces 5",
            "-repeat",
            &format!("-rounds {}", num_games),
            "-games 2",
            "-draw movenumber=40 movecount=4 score=8",
            "-resign movecount=4 score=500",
            "-ratinginterval 1",
            &format!("-sprt elo0={} elo1={} alpha={:.3} beta={:.3}",
                    elo0, elo1, confidence, confidence),
        ];

        let args = args.into_iter()
            .flat_map(|arg| arg.split_ascii_whitespace())
            .collect::<Vec<_>>();

        let (tx,rx) = crossbeam::channel::unbounded();

        let mut child: Child = Command::new("cutechess-cli")
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .expect("failed to spawn cutechess-cli");

        std::thread::sleep(std::time::Duration::from_millis(10));
        let pid = child.id();
        eprintln!("pid = {:?}", pid);

        let children = Command::new("pgrep")
            .args(["--parent", &format!("{}", pid)])
            .output().unwrap();

        let children = String::from_utf8(children.stdout).unwrap()
            .lines()
            .map(|line| u32::from_str(line).unwrap())
            .collect();

        eprintln!("children = {:?}", children);

        let cutechess = CuteChess {
            // child:    Arc::new(child),
            pid,
            children,
            stop:     Arc::new(AtomicBool::new(false)),
            rx:       Arc::new(rx),
        };

        // let pid = cutechess.id();
        // eprintln!("pid = {:?}", pid);

        let cutechess2 = cutechess.clone();
        std::thread::spawn(move || {
            cutechess2._run_cutechess(child,tx);
        });

        cutechess
    }

    fn _run_cutechess(&self, child: Child, tx: Sender<Match>) {
        println!("starting _run_cutechess, pid = {:>5}", self.pid);

        let mut game: Vec<String>   = vec![];
        // let mut matches: Vec<Match> = vec![];

        let mut state = InputParser::None;

        let reader = BufReader::new(child.stdout.unwrap());

        for line in reader.lines() {

            if self.stop.load(std::sync::atomic::Ordering::SeqCst) {
                println!("exiting _run_cutechess, pid = {:>5}", self.pid);
                break;
            }

            let line = line.unwrap();

            match state {
                InputParser::None => {
                    game.push(line);
                    state = InputParser::Started;
                },
                InputParser::Started => {
                    game.push(line.clone());

                    if line.starts_with("Elo difference") {
                        let v = std::mem::replace(&mut game, vec![]);

                        // for line in v {
                        //     eprintln!("{:?}", line);
                        // }

                        let res = Match::parse(v).unwrap();

                        // eprintln!("{:?}", res);

                        // matches.push(res);
                        tx.send(res).unwrap();

                        state = InputParser::Started;
                    }
                },
            }

        }
    }

}

pub fn get_timestamp() -> String {
    use chrono::{Datelike,Timelike};
    let now = chrono::Local::now();
    format!("{:0>4}-{:0>2}-{:0>2}_{:0>2}:{:0>2}:{:0>2}",
            now.year(), now.month(), now.day(),
            now.hour(), now.minute(), now.second())
}

