
use crate::sprt::sprt_penta::SPRT;
use crate::tuner_types::*;
use crate::json_config::Engine;

use once_cell::sync::Lazy;
use rchess_engine_lib::explore::AtomicBool;

use std::str::FromStr;
use std::io::BufReader;
use std::io::{self,Write,BufRead,Stdout,Stdin};
use std::process::{Command,Stdio, Child};
use std::sync::Arc;

use crossbeam::channel::{Sender,Receiver};

use derive_new::new;

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
    pub engine_tuning:     Engine,
    pub engine_baseline:   Engine,

    pub tunable:           Tunable,

    pub timecontrol:       TimeControl,

    pub t0:                std::time::Instant,
    pub sprts:             Vec<(i32,SPRT)>,

    tx_rx:                 Option<(Arc<Sender<MatchOutcome>>,Arc<Receiver<MatchOutcome>>)>,

    pub brackets:          [f64; 2],
    // pub hyps:              Vec<f64>,
    // pub hyp_accepted:      Vec<f64>,
    // pub hyp_rejected:      Vec<f64>,
}

impl Supervisor {
    pub fn new(
        engine_tuning:      Engine,
        engine_baseline:    Engine,
        tunable:            Tunable,
        timecontrol:        TimeControl,
    ) -> Self {

        let mut sprts = vec![];

        // for elo in [0.,5.,10.,15.,20.,30.,40.,50.,60.,80.,100.,150.,200.].iter().rev() {
        //     let elo = -elo;
        //     sprts.push((elo as i32, SPRT::new(0., elo, 0.05, 0.05)));
        // }

        // for elo in [0.,5.,10.,15.,20.,30.,40.,50.,60.,80.,100.,150.,200.] {
        //     sprts.push((elo as i32, SPRT::new(0., elo, 0.05)));
        // }

        for elo in 0..50 {
            sprts.push((elo as i32, SPRT::new(0., elo as f64, 0.05)));
        }

        Self {
            engine_tuning,
            engine_baseline,
            tunable,
            timecontrol,
            t0:             std::time::Instant::now(),
            sprts,
            tx_rx:          None,
            brackets:       [0.0; 2],
            // hyps:           vec![0.,5.,10.,15.,20.,30.,40.,50.,60.,80.,100.,150.,200.],
            // hyp_accepted:   vec![],
            // hyp_rejected:   vec![],
        }
    }

    pub fn get_rx(&self) -> Arc<Receiver<MatchOutcome>> {
        self.tx_rx.as_ref().unwrap().1.clone()
    }

    pub fn tx_rx(&mut self) -> (Arc<Sender<MatchOutcome>>,Arc<Receiver<MatchOutcome>>) {
        if let Some((tx,rx)) = self.tx_rx.as_ref() {
            (tx.clone(),rx.clone())
        } else {
            let (tx,rx) = crossbeam::channel::unbounded();
            let tx = Arc::new(tx);
            let rx = Arc::new(rx);
            self.tx_rx = Some((tx.clone(),rx.clone()));
            (tx,rx)
        }
    }

}

#[derive(Debug,PartialEq,Eq,PartialOrd,Ord,Clone)]
pub struct TunableOpt {
    pub name:    String,
    pub min:     u64,
    pub max:     u64,
    pub start:   u64,
    pub step:    u64,
}

#[derive(Debug,Clone)]
pub struct Tunable {
    pub opt:            TunableOpt,
    // pub attempts:   Vec<(u64, Hypothesis)>,
}

impl Tunable {
    pub fn new(name: String, min: u64, max: u64, start: u64, step: u64) -> Self {
        Self {
            opt: TunableOpt { name, min, max, start, step},
            // attempts: vec![],
        }
    }
}

#[derive(Debug,Clone)]
pub struct CuteChess {
    // pub child:       Arc<Option<Child>>,
    pub pid:         u32,
    pub children:    Vec<u32>,
    pub stop:        Arc<AtomicBool>,
    // pub rx:          Arc<Receiver<MatchOutcome>>,
}

impl Drop for CuteChess {
    fn drop(&mut self) {
        for child in self.children.iter() {
            nix::sys::signal::kill(
                nix::unistd::Pid::from_raw(*child as i32), nix::sys::signal::SIGKILL)
                .unwrap_or_else(|_| {});
        }
        nix::sys::signal::kill(
            nix::unistd::Pid::from_raw(self.pid as i32), nix::sys::signal::SIGKILL)
            .unwrap_or_else(|_| {});
    }
}

/// kill
impl CuteChess {
    pub fn kill(self) {}
}

/// run
impl CuteChess {

    pub fn run_cutechess(
        rx:               Arc<Receiver<MatchOutcome>>,
        tx:               Arc<Sender<MatchOutcome>>,
        engine1:          &str,
        engine2:          &str,
        timecontrol:      TimeControl,
        output_label:     &str,
        num_games:        u64,
    ) -> CuteChess {
        let timestamp   = get_timestamp();
        let output_file = &format!("tuning_logs/out_{}_{}.pgn", output_label, timestamp);
        let log_file    = &format!("tuning_logs/log_{}_{}.pgn", output_label, timestamp);
        let time        = timecontrol.print();
        let margin      = 10;

        let args = [
            "-tournament gauntlet",
            "-concurrency 1",
            &format!("-pgnout {}", output_file),
            &format!("-engine conf={} {} timemargin={margin} restart=off", engine1, time),
            &format!("-engine conf={} {} timemargin={margin} restart=off", engine2, time),
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
        ];

        let args = args.into_iter()
            .flat_map(|arg| arg.split_ascii_whitespace())
            .collect::<Vec<_>>();

        Self::start_cutechess(rx, tx, &args, log_file)
    }

    pub fn run_cutechess_tournament(
        rx:               Arc<Receiver<MatchOutcome>>,
        tx:               Arc<Sender<MatchOutcome>>,
        engine1:          &str,
        engine2:          &str,
        timecontrol:      TimeControl,
        output_label:     &str,
        num_games:        u64,
        (elo0,elo1):      (u64,u64),
        confidence:       f64,
    ) -> CuteChess {
        let timestamp   = get_timestamp();
        let output_file = &format!("tuning_logs/out_{}_{}.pgn", output_label, timestamp);
        let log_file    = &format!("tuning_logs/log_{}_{}.pgn", output_label, timestamp);
        let time        = timecontrol.print();
        let margin      = 10;

        let args = [
            "-tournament gauntlet",
            "-concurrency 1",
            &format!("-pgnout {}", output_file),
            &format!("-engine conf={} {} timemargin={margin} restart=off", engine1, time),
            &format!("-engine conf={} {} timemargin={margin} restart=off", engine2, time),
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

        Self::start_cutechess(rx, tx, &args, log_file)
    }

    fn start_cutechess(
        rx:         Arc<Receiver<MatchOutcome>>,
        tx:         Arc<Sender<MatchOutcome>>,
        args:       &[&str],
        log_file:   &str,
    ) -> CuteChess {
        // let (tx,rx) = crossbeam::channel::unbounded();

        let mut child: Child = Command::new("cutechess-cli")
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .expect("failed to spawn cutechess-cli");


        let pid = child.id();
        debug!("cutechess pid = {:?}", pid);

        let mut children = vec![];
        while children.len() < 2 {
            std::thread::sleep(std::time::Duration::from_millis(10));

            let pgrep = Command::new("pgrep")
                .args(["--parent", &format!("{}", pid)])
                .output().unwrap();

            children = String::from_utf8(pgrep.stdout).unwrap()
                .lines()
                .map(|line| u32::from_str(line).unwrap())
                .collect();
        }

        debug!("cutechess children = {:?}", children);

        let cutechess = CuteChess {
            // child:    Arc::new(child),
            pid,
            children,
            stop:     Arc::new(AtomicBool::new(false)),
            // rx:       rx,
        };

        // let pid = cutechess.id();
        // eprintln!("pid = {:?}", pid);

        let cutechess2 = cutechess.clone();
        std::thread::spawn(move || {
            cutechess2.listen_cutechess(child,tx);
        });

        cutechess
    }

    fn listen_cutechess(&self, child: Child, tx: Arc<Sender<MatchOutcome>>) {
        debug!("starting listen_cutechess, pid = {:>5}", self.pid);

        let mut game: Vec<String>   = vec![];
        // let mut matches: Vec<Match> = vec![];

        let mut state = InputParser::None;

        let reader = BufReader::new(child.stdout.unwrap());

        let mut pair = vec![];

        for line in reader.lines() {

            // trace!("line = {:?}", &line);
            trace!("line = {:?}", &line);
            let line = line.unwrap();

            if self.stop.load(std::sync::atomic::Ordering::SeqCst) {
                debug!("exiting listen_cutechess, pid = {:>5}", self.pid);
                break;
            }

            match state {
                InputParser::None => {
                    game.push(line);
                    state = InputParser::Started;
                },

                InputParser::Started => {
                    if line.starts_with("Started game") {
                        let v = std::mem::replace(&mut game, vec![]);
                        let res = MatchOutcome::parse(v).unwrap();

                        pair.push(res);

                        game.push(line);
                    } else {
                        game.push(line);
                    }
                },
            }

            if pair.len() == 2 {

                match (pair[0], pair[1]) {
                    (MatchOutcome::Match(m0),MatchOutcome::Match(m1)) => {
                        tx.send(MatchOutcome::MatchPair(m0, m1)).unwrap();
                        pair.clear();
                    },
                    _ => panic!(),
                }
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

