#![allow(dead_code, unused_variables)]
use crate::common_map::do_map;
use crate::common_reduce::do_reduce;
use crate::utils::*;
use crate::wc;
use crate::master_splitmerge::merge;

use std::thread;

use log::info;

pub use crate::master_rpc::distribucted as start_master_server;

pub fn run<F: Fn(JobPhase)>(
    job_name: String,
    files: &Vec<String>,
    n_reduce: usize,
    schedule:F,
    finish: fn(),
) {
    info!("Start run: ");
    schedule(JobPhase::MapPhase);
    schedule(JobPhase::ReducePhase);
    finish();
    merge(&job_name, n_reduce);
    info!("Run finish");
}

#[allow(non_snake_case)]
pub fn sequential(
    job_name: String,
    files: Vec<String>,
    n_reduce: usize,
    mapF: fn(&str, &str) -> Vec<KeyValue>,
    reduceF: fn(&str, &Vec<String>) -> String,
) {
    let handle = thread::spawn(move || {
        run(
            job_name.clone(),
            &files,
            n_reduce,
            |phase| match phase {
                JobPhase::MapPhase => {
                    for (i, f) in files.iter().enumerate() {
                        do_map(&job_name, i, f, n_reduce, mapF);
                    }
                }
                JobPhase::ReducePhase => {
                    for i in 0..n_reduce {
                        do_reduce(
                            &job_name,
                            i,
                            &merge_name(&job_name, i),
                            files.len(),
                            reduceF,
                        );
                    }
                }
            },
            seq_finish,
        )
    });
    handle.join().expect("sequential join failed");
}

fn seq_finish() {
    info!("check the mrtmp.{{jobname}} file for result");
}

fn wc_seq() {
    sequential(
        "test".to_owned(),
        vec!["test".to_owned()],
        1,
        wc::map,
        wc::reduce,
    )
}
